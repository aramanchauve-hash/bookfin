use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use bookfin::application::dtos::SubmitReactionRequestDto;
use bookfin::application::ports::{ImpressionRepository, PageRepository, ReactionRepository};
use bookfin::application::use_cases::{
    ContinueReadingUseCase, RevealPageMetadataUseCase, SubmitReactionUseCase,
};
use bookfin::domain::errors::DomainError;
use bookfin::domain::models::{
    sanitize_reading_time, sanitize_scroll_depth, validate_reading_metrics_full,
    validate_server_timing_with_config, Page, ReadingValidationConfig,
};
use bookfin::infrastructure::db::init_db_pool;
use bookfin::infrastructure::repositories::{
    PostgresImpressionRepository, PostgresPageRepository, PostgresReactionRepository,
};

async fn get_test_pool() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());
    init_db_pool(&url).await.ok()
}

// --------------------------------------------------
// Section 7: Validation de lecture (Cas A, B, C, D, E)
// --------------------------------------------------

#[test]
fn test_reading_validation_cases_a_to_e() {
    let config = ReadingValidationConfig::default();

    // Cas A : Page entièrement visible, content_overflows = false, aucun scroll, temps suffisant
    // -> réaction valide
    let res_a = validate_reading_metrics_full(4200, 0.0, false, false, &config);
    assert!(res_a.is_ok(), "Cas A doit être valide sans scroll");

    // Cas B : Page avec overflow, bottom_reached = true, scroll intermédiaire
    // -> réaction valide
    let res_b = validate_reading_metrics_full(5000, 0.50, true, true, &config);
    assert!(
        res_b.is_ok(),
        "Cas B doit être valide si le bas a été atteint"
    );

    // Cas C : Page avec overflow, scroll insuffisant (0.60 < 0.75), bottom non atteint
    // -> réaction refusée
    let res_c = validate_reading_metrics_full(5000, 0.60, false, true, &config);
    assert!(
        matches!(res_c, Err(DomainError::InsufficientScroll { .. })),
        "Cas C doit être rejeté pour scroll insuffisant"
    );

    // Cas D : Temps actif client falsifié (ex: 8000ms), mais temps serveur écoulé insuffisant (ex: 2000ms < 3500ms)
    // -> réaction refusée
    let now = Utc::now();
    let served_at = now - Duration::milliseconds(2000);
    let res_d = validate_server_timing_with_config(served_at, now, &config);
    assert!(
        matches!(
            res_d,
            Err(DomainError::ServerTimeElapsedTooShort {
                elapsed_ms,
                min_required_ms
            }) if elapsed_ms < 3500 && min_required_ms == 3500
        ),
        "Cas D doit être rejeté si le temps serveur est trop court"
    );

    // Cas E : Temps extrêmement long (téléphone laissé ouvert : ex: 10 heures = 36 000 000 ms)
    // -> données acceptées mais correctement bornées/nettoyées à max_reading_time_ms (1 800 000 ms = 30 min)
    let sanitized_time = sanitize_reading_time(36_000_000, &config);
    assert_eq!(
        sanitized_time, config.max_reading_time_ms,
        "Cas E : le temps excessif doit être borné à max_reading_time_ms"
    );

    let sanitized_scroll_excess = sanitize_scroll_depth(1.85);
    assert_eq!(
        sanitized_scroll_excess, 1.0,
        "Le scroll supérieur à 1.0 doit être borné à 1.0"
    );
    let sanitized_scroll_neg = sanitize_scroll_depth(-0.4);
    assert_eq!(
        sanitized_scroll_neg, 0.0,
        "Le scroll négatif doit être borné à 0.0"
    );
}

// --------------------------------------------------
// Régression bug alpha : 422 métriques de lecture bloquant côté mobile
// --------------------------------------------------

#[test]
fn test_reading_validation_reason_is_stable_and_machine_readable() {
    use bookfin::web::api_v1::reading_validation_reason;

    let config = ReadingValidationConfig::default();

    let too_fast = validate_reading_metrics_full(1000, 1.0, true, false, &config).unwrap_err();
    assert_eq!(
        reading_validation_reason(&too_fast),
        Some("reading_too_fast")
    );

    let insufficient_scroll =
        validate_reading_metrics_full(5000, 0.10, false, true, &config).unwrap_err();
    assert_eq!(
        reading_validation_reason(&insufficient_scroll),
        Some("insufficient_scroll")
    );

    let now = Utc::now();
    let served_at = now - Duration::milliseconds(500);
    let too_soon = validate_server_timing_with_config(served_at, now, &config).unwrap_err();
    assert_eq!(
        reading_validation_reason(&too_soon),
        Some("server_time_elapsed_too_short")
    );

    // Une erreur qui n'est pas une validation de lecture ne doit jamais recevoir de raison
    // "récupérable" : le mobile doit la traiter comme une vraie erreur (réseau/serveur).
    assert_eq!(
        reading_validation_reason(&DomainError::AlreadyReacted),
        None
    );
}

#[test]
fn test_scrollable_page_becomes_valid_after_real_progression() {
    // Contenu qui dépasse le viewport (content_overflows = true) sur Android :
    // un simple survol ne suffit pas, il faut une progression réelle du scroll
    // pour que la réaction soit validée (bas de page non atteint).
    let config = ReadingValidationConfig::default();

    // Progression insuffisante (l'utilisateur vient d'ouvrir la page) -> rejeté
    let barely_scrolled = validate_reading_metrics_full(5000, 0.05, false, true, &config);
    assert!(
        matches!(barely_scrolled, Err(DomainError::InsufficientScroll { .. })),
        "Un scroll quasi nul sur une page qui déborde doit être rejeté"
    );

    // Progression réelle jusqu'au seuil requis -> accepté
    let fully_scrolled = validate_reading_metrics_full(5000, 0.80, false, true, &config);
    assert!(
        fully_scrolled.is_ok(),
        "Une progression réelle de scroll (0.80 >= seuil 0.75) doit valider la lecture"
    );

    // Atteindre le bas de la page dispense explicitement du seuil de profondeur
    let bottom_reached_early = validate_reading_metrics_full(5000, 0.05, true, true, &config);
    assert!(
        bottom_reached_early.is_ok(),
        "bottom_reached=true doit valider la lecture même avec un scroll_depth faible"
    );
}

// --------------------------------------------------
// Section 1, 2, 4, 8: Tests d'intégration sur base de données
// --------------------------------------------------

#[tokio::test]
async fn test_hardening_reaction_requires_prior_impression() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let page_id = Uuid::new_v4();

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool);
    let use_case = SubmitReactionUseCase::new(reaction_repo, impression_repo);
    let config = ReadingValidationConfig::default();

    // Tenter de réagir à une page sans impression préalable
    let req = SubmitReactionRequestDto {
        event_id: Some(Uuid::new_v4()),
        user_id,
        page_id: Some(page_id),
        extract_id: None,
        impression_id: None,
        reaction: "like".to_string(),
        reading_time_ms: 5000,
        scroll_depth: 0.9,
        bottom_reached: Some(true),
        content_overflows: Some(true),
        navigation_action: None,
        served_at: None,
        reacted_at: None,
        app_version: None,
        build_version: None,
    };

    let result = use_case.execute(req, &config).await;
    assert!(
        matches!(result, Err(DomainError::ImpressionNotFound(_))),
        "Une réaction sans impression préalable doit échouer avec ImpressionNotFound"
    );
}

#[tokio::test]
async fn test_hardening_single_reaction_per_impression_and_idempotency() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(work_id)
    .bind("Titre Hardening")
    .bind("Auteur Hardening")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(edition_id)
    .bind(work_id)
    .bind("Édition Hardening")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    let hash = Page::compute_hash("Texte hardening");
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, 'Texte hardening', $3, 'fr', 20, 0.4)",
    )
    .bind(page_id)
    .bind(edition_id)
    .bind(&hash)
    .execute(&pool)
    .await
    .unwrap();

    let impression_repo = PostgresImpressionRepository::new(pool.clone());
    let reaction_repo = PostgresReactionRepository::new(pool.clone());

    // Créer une impression servie il y a 5 secondes
    let impression = impression_repo
        .record_impression(user_id, page_id, None)
        .await
        .unwrap();

    // Mettre à jour served_at pour satisfaire le délai serveur de 3500ms
    let past_time = Utc::now() - Duration::milliseconds(4500);
    sqlx::query("UPDATE page_impressions SET served_at = $1 WHERE id = $2")
        .bind(past_time)
        .bind(impression.id)
        .execute(&pool)
        .await
        .unwrap();

    let use_case = SubmitReactionUseCase::new(reaction_repo, impression_repo);
    let config = ReadingValidationConfig::default();
    let event_id = Uuid::new_v4();

    // 1. Première réaction (LIKE)
    let req1 = SubmitReactionRequestDto {
        event_id: Some(event_id),
        user_id,
        page_id: Some(page_id),
        extract_id: None,
        impression_id: Some(impression.id),
        reaction: "like".to_string(),
        reading_time_ms: 4500,
        scroll_depth: 0.9,
        bottom_reached: Some(true),
        content_overflows: Some(true),
        navigation_action: None,
        served_at: None,
        reacted_at: None,
        app_version: None,
        build_version: None,
    };

    let res1 = use_case.execute(req1, &config).await.expect("Première réaction réussie");
    assert!(res1.success);
    assert!(res1.recorded, "La première réaction doit être enregistrée");

    // 2. Retry avec le MÊME event_id (idempotence réseau)
    let req_retry = SubmitReactionRequestDto {
        event_id: Some(event_id),
        user_id,
        page_id: Some(page_id),
        extract_id: None,
        impression_id: Some(impression.id),
        reaction: "like".to_string(),
        reading_time_ms: 4500,
        scroll_depth: 0.9,
        bottom_reached: Some(true),
        content_overflows: Some(true),
        navigation_action: None,
        served_at: None,
        reacted_at: None,
        app_version: None,
        build_version: None,
    };

    let res_retry = use_case.execute(req_retry, &config).await.expect("Retry idempotent accepté");
    assert!(res_retry.success);
    assert!(!res_retry.recorded, "Le retry ne doit pas insérer une nouvelle ligne");
    assert_eq!(res_retry.reaction_id, res1.reaction_id, "L'id de réaction doit être identique");

    // 3. Deuxième réaction différente sur la MÊME impression (double tap / conflit)
    let req_conflict = SubmitReactionRequestDto {
        event_id: Some(Uuid::new_v4()), // Nouvel event_id
        user_id,
        page_id: Some(page_id),
        extract_id: None,
        impression_id: Some(impression.id),
        reaction: "dislike".to_string(), // Tente d'écraser par DISLIKE
        reading_time_ms: 5000,
        scroll_depth: 0.95,
        bottom_reached: Some(true),
        content_overflows: Some(true),
        navigation_action: None,
        served_at: None,
        reacted_at: None,
        app_version: None,
        build_version: None,
    };

    let res_conflict = use_case.execute(req_conflict, &config).await;
    assert_eq!(
        res_conflict,
        Err(DomainError::AlreadyReacted),
        "Une deuxième réaction sur la même impression doit être rejetée avec AlreadyReacted"
    );
}

#[tokio::test]
async fn test_hardening_reveal_forbidden_before_reaction() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(work_id)
    .bind("Titre Secret")
    .bind("Auteur Secret")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(edition_id)
    .bind(work_id)
    .bind("Édition Secrète")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    let hash = Page::compute_hash("Texte secret");
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, 'Texte secret', $3, 'fr', 20, 0.4)",
    )
    .bind(page_id)
    .bind(edition_id)
    .bind(&hash)
    .execute(&pool)
    .await
    .unwrap();

    let page_repo = PostgresPageRepository::new(pool.clone());
    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let use_case = RevealPageMetadataUseCase::new(page_repo, reaction_repo);

    // 1. Avant réaction -> DOIT être rejeté avec ReactionRequiredForReveal
    let reveal_err = use_case.execute(user_id, page_id).await;
    assert_eq!(
        reveal_err,
        Err(DomainError::ReactionRequiredForReveal),
        "La métadonnée ne doit jamais être révélée avant réaction"
    );

    // 2. Enregistrer une réaction
    let now = Utc::now();
    let reaction = bookfin::domain::models::Reaction {
        id: Uuid::new_v4(),
        event_id: Uuid::new_v4(),
        user_id,
        page_id,
        impression_id: None,
        reaction_type: bookfin::domain::models::ReactionType::Like,
        reading_time_ms: 4500,
        scroll_depth: 0.9,
        bottom_reached: true,
        content_overflows: false,
        navigation_action: None,
        served_at: now,
        reacted_at: now,
        created_at: now,
        app_version: None,
        build_version: None,
    };
    PostgresReactionRepository::new(pool.clone())
        .record_reaction(&reaction)
        .await
        .unwrap();

    // 3. Après réaction -> Révélation autorisée
    let reveal_ok = use_case.execute(user_id, page_id).await.expect("Révélation autorisée");
    assert_eq!(reveal_ok.title, "Titre Secret");
    assert_eq!(reveal_ok.author, "Auteur Secret");
    assert!(reveal_ok.has_reacted);
}

#[tokio::test]
async fn test_hardening_continue_end_of_edition() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page_last_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(work_id)
    .bind("Livre Court")
    .bind("Auteur")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(edition_id)
    .bind(work_id)
    .bind("Édition Unique")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    // Seule page (dernière page = page 1)
    let hash = Page::compute_hash("Fin du livre");
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, 'Fin du livre', $3, 'fr', 20, 0.4)",
    )
    .bind(page_last_id)
    .bind(edition_id)
    .bind(&hash)
    .execute(&pool)
    .await
    .unwrap();

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool.clone());
    
    // Enregistrement d'une impression préalable
    let imp = impression_repo.record_impression(user_id, page_last_id, None).await.unwrap();
    assert!(!imp.reached_end, "reached_end doit être initialement false");

    let use_case = ContinueReadingUseCase::new(page_repo, impression_repo.clone());

    let res = use_case
        .execute(user_id, page_last_id, Some(imp.id), None, None, None)
        .await;
    assert_eq!(
        res,
        Err(DomainError::EndOfEdition {
            edition_id,
            last_page_number: 1,
        }),
        "Continuer après la dernière page doit renvoyer EndOfEdition"
    );

    // Vérifier que le signal reached_end est bien persisté à true en base
    let imp_after = impression_repo.get_impression_by_id(imp.id).await.unwrap().unwrap();
    assert!(imp_after.reached_end, "L'impression parente doit être marquée reached_end = true");
}

#[tokio::test]
async fn test_hardening_continuation_chain_reconstruction() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let p1_id = Uuid::new_v4();
    let p2_id = Uuid::new_v4();
    let p3_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(work_id)
    .bind("Trilogie")
    .bind("Auteur")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(edition_id)
    .bind(work_id)
    .bind("Édition Chaîne")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    let h10 = Page::compute_hash("Page 10");
    let h11 = Page::compute_hash("Page 11");
    let h12 = Page::compute_hash("Page 12");
    sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 10, 'Page 10', $3, 'fr', 20, 0.1)")
        .bind(p1_id).bind(edition_id).bind(&h10).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 11, 'Page 11', $3, 'fr', 20, 0.2)")
        .bind(p2_id).bind(edition_id).bind(&h11).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 12, 'Page 12', $3, 'fr', 20, 0.3)")
        .bind(p3_id).bind(edition_id).bind(&h12).execute(&pool).await.unwrap();

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool.clone());

    // 1. Tirage initial aléatoire sur page 10 (depth = 0)
    let imp1 = impression_repo.record_impression(user_id, p1_id, None).await.unwrap();
    assert_eq!(imp1.continuation_depth, 0);
    assert_eq!(imp1.parent_impression_id, None);

    // 2. Continuer vers page 11 (depth = 1)
    let continue_use_case = ContinueReadingUseCase::new(page_repo.clone(), impression_repo.clone());
    let step2 = continue_use_case
        .execute(user_id, p1_id, Some(imp1.id), None, None, None)
        .await
        .expect("Continue vers page 11");
    assert_eq!(step2.page_id, p2_id);
    assert_eq!(step2.continuation_depth, 1);

    // 3. Retry réseau sur la même requête (idempotence)
    let step2_retry = continue_use_case
        .execute(user_id, p1_id, Some(imp1.id), None, None, None)
        .await
        .expect("Retry continue vers page 11");
    assert_eq!(step2_retry.impression_id, step2.impression_id, "Le retry doit renvoyer la même impression");

    // 4. Continuer vers page 12 (depth = 2)
    let step3 = continue_use_case
        .execute(user_id, p2_id, Some(step2.impression_id), None, None, None)
        .await
        .expect("Continue vers page 12");
    assert_eq!(step3.page_id, p3_id);
    assert_eq!(step3.continuation_depth, 2);

    // 5. Reconstruction récursive de la chaîne de continuation
    let chain = impression_repo
        .get_continuation_chain(step3.impression_id)
        .await
        .expect("Reconstruction de la chaîne");
    assert_eq!(chain.len(), 3, "La chaîne doit contenir 3 impressions");
    assert_eq!(chain[0].id, imp1.id);
    assert_eq!(chain[0].continuation_depth, 0);
    assert_eq!(chain[1].id, step2.impression_id);
    assert_eq!(chain[1].continuation_depth, 1);
    assert_eq!(chain[1].parent_impression_id, Some(imp1.id));
    assert_eq!(chain[2].id, step3.impression_id);
    assert_eq!(chain[2].continuation_depth, 2);
    assert_eq!(chain[2].parent_impression_id, Some(step2.impression_id));
}

#[tokio::test]
async fn test_hardening_immutability_trigger_prohibits_mutation() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 4").execute(&pool).await;
    let mig_res = sqlx::migrate!("./migrations").run(&pool).await;
    println!("Migrate result in test: {:?}", mig_res);
    let tg: Option<(String,)> = sqlx::query_as("SELECT tgname::text FROM pg_trigger WHERE tgname = 'trg_prevent_page_content_mutation'")
        .fetch_optional(&pool)
        .await
        .unwrap();
    println!("Trigger found: {:?}", tg);

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(work_id)
    .bind("Livre Immuable")
    .bind("Auteur")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(edition_id)
    .bind(work_id)
    .bind("Édition Immuable")
    .bind("fr")
    .execute(&pool)
    .await
    .unwrap();

    let hash = Page::compute_hash("Texte original immuable");
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, 'Texte original immuable', $3, 'fr', 20, 0.5)",
    )
    .bind(page_id)
    .bind(edition_id)
    .bind(&hash)
    .execute(&pool)
    .await
    .unwrap();

    // 1. Modification sur une page jamais servie : AUTORISÉE (corrections avant publication)
    let pre_update = sqlx::query("UPDATE pages SET content = 'Texte corrigé avant publication' WHERE id = $1")
        .bind(page_id)
        .execute(&pool)
        .await;
    assert!(pre_update.is_ok(), "Mise à jour autorisée si la page n'a jamais été servie");

    // 2. Servir la page (création d'une impression historique)
    let impression_repo = PostgresImpressionRepository::new(pool.clone());
    impression_repo.record_impression(user_id, page_id, None).await.unwrap();

    // 3. Modification après service historique : STRICTEMENT INTERDITE par le trigger
    let post_update = sqlx::query("UPDATE pages SET content = 'Tentative d''altération historique' WHERE id = $1")
        .bind(page_id)
        .execute(&pool)
        .await;

    assert!(
        post_update.is_err(),
        "Le trigger doit interdire toute modification de texte d'une page ayant une impression historique"
    );
    let err_msg = post_update.unwrap_err().to_string();
    assert!(
        err_msg.contains("ImmutablePageViolation"),
        "L'erreur doit mentionner ImmutablePageViolation (reçu: {})",
        err_msg
    );
}

#[tokio::test]
async fn test_random_page_strictly_excludes_seen_pages() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page_a = Uuid::new_v4();
    let page_b = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)").bind(user_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, 'W', 'A', 'fr')")
        .bind(work_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, 'E', 'fr')")
        .bind(edition_id).bind(work_id).execute(&pool).await.unwrap();

    let ha = Page::compute_hash("Page A content");
    let hb = Page::compute_hash("Page B content");
    sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, 'Page A content', $3, 'fr', 10, 0.1)")
        .bind(page_a).bind(edition_id).bind(&ha).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 2, 'Page B content', $3, 'fr', 10, 0.9)")
        .bind(page_b).bind(edition_id).bind(&hb).execute(&pool).await.unwrap();

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool.clone());

    // 1. Avant toute impression : les deux pages sont éligibles
    let first_draw = page_repo.get_unseen_random(user_id, 0.05).await.unwrap();
    assert!(first_draw.is_some());

    // 2. Marquer Page A comme vue
    impression_repo.record_impression(user_id, page_a, None).await.unwrap();

    // 3. Tirage aléatoire : Page A doit être STRICTEMENT exclue
    for _ in 0..5 {
        let draw = page_repo.get_unseen_random(user_id, 0.0).await.unwrap().expect("Une page non vue disponible");
        assert_ne!(draw.id, page_a, "Page A déjà vue ne doit plus jamais être servie par RANDOM_PAGE");
    }

    // 4. Marquer TOUTES les pages existantes comme vues pour cet utilisateur
    sqlx::query("INSERT INTO page_impressions (id, user_id, page_id, served_at) SELECT gen_random_uuid(), $1, id, NOW() FROM pages")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    // 5. Tirage aléatoire : plus aucune page éligible non vue -> None
    let exhausted = page_repo.get_unseen_random(user_id, 0.0).await.unwrap();
    assert!(exhausted.is_none(), "RANDOM_PAGE doit retourner None quand toutes les pages ont été vues");

    // 6. En revanche, CONTINUE_BOOK depuis Page 1 peut volontairement resservir Page 2 même si déjà vue
    let continue_use_case = ContinueReadingUseCase::new(page_repo, impression_repo);
    let continued = continue_use_case.execute(user_id, page_a, None, None, None, None).await;
    assert!(continued.is_ok(), "CONTINUE_BOOK a le droit légitime de resservir une page pour continuer l'œuvre");
    assert_eq!(continued.unwrap().page_id, page_b);
}

#[test]
fn test_json_contract_feed_never_leaks_metadata() {
    use bookfin::application::dtos::FeedPageDto;

    let dto = FeedPageDto {
        impression_id: Uuid::new_v4(),
        page_id: Uuid::new_v4(),
        page_sequence_number: 42,
        source_page_number: Some("XLII".to_string()),
        text: "Le vent se lève, il faut tenter de vivre.".to_string(),
        language_tag: "fr".to_string(),
        token_count: 8,
        continuation_depth: 0,
        served_at: Utc::now(),
    };

    let json_val = serde_json::to_value(&dto).expect("Sérialisation JSON");
    let obj = json_val.as_object().expect("Objet JSON");

    // Champs autorisés
    assert!(obj.contains_key("impression_id"));
    assert!(obj.contains_key("page_id"));
    assert!(obj.contains_key("page_sequence_number"));
    assert!(obj.contains_key("source_page_number"));
    assert!(obj.contains_key("text"));
    assert!(obj.contains_key("language_tag"));
    assert!(obj.contains_key("token_count"));
    assert!(obj.contains_key("continuation_depth"));
    assert!(obj.contains_key("served_at"));

    // Invariant fondamental de confidentialité pré-réaction : INTERDICTION absolue de fuite
    assert!(!obj.contains_key("title"), "Le titre ne doit JAMAIS fuiter dans le feed");
    assert!(!obj.contains_key("author"), "L'auteur ne doit JAMAIS fuiter dans le feed");
    assert!(!obj.contains_key("work_id"), "Le work_id ne doit JAMAIS fuiter dans le feed");
    assert!(!obj.contains_key("edition_title"), "Le titre d'édition ne doit pas fuiter");
    assert!(!obj.contains_key("translator"), "Le traducteur ne doit pas fuiter");
    assert!(!obj.contains_key("publication_year"), "L'année ne doit pas fuiter");
    assert!(!obj.contains_key("source_name"), "Le nom de source ne doit pas fuiter");
}

#[tokio::test]
async fn test_social_privacy_isolated_stats() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1), ($2)").bind(user_a).bind(user_b).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, 'WP', 'AP', 'fr')")
        .bind(work_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, 'EP', 'fr')")
        .bind(edition_id).bind(work_id).execute(&pool).await.unwrap();
    let hp = Page::compute_hash("Privacy test page");
    sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, 'Privacy test page', $3, 'fr', 10, 0.5)")
        .bind(page_id).bind(edition_id).bind(&hp).execute(&pool).await.unwrap();

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let now = Utc::now();

    // User A a 1 like
    let r_a = bookfin::domain::models::Reaction {
        id: Uuid::new_v4(),
        event_id: Uuid::new_v4(),
        user_id: user_a,
        page_id,
        impression_id: None,
        reaction_type: bookfin::domain::models::ReactionType::Like,
        reading_time_ms: 5000,
        scroll_depth: 0.9,
        bottom_reached: true,
        content_overflows: false,
        navigation_action: None,
        served_at: now,
        reacted_at: now,
        created_at: now,
        app_version: None,
        build_version: None,
    };
    reaction_repo.record_reaction(&r_a).await.unwrap();

    // User B a 1 dislike
    let r_b = bookfin::domain::models::Reaction {
        id: Uuid::new_v4(),
        event_id: Uuid::new_v4(),
        user_id: user_b,
        page_id,
        impression_id: None,
        reaction_type: bookfin::domain::models::ReactionType::Dislike,
        reading_time_ms: 5000,
        scroll_depth: 0.9,
        bottom_reached: true,
        content_overflows: false,
        navigation_action: None,
        served_at: now,
        reacted_at: now,
        created_at: now,
        app_version: None,
        build_version: None,
    };
    reaction_repo.record_reaction(&r_b).await.unwrap();

    let stats_a = reaction_repo.get_user_reading_stats(user_a).await.unwrap();
    let stats_b = reaction_repo.get_user_reading_stats(user_b).await.unwrap();

    assert_eq!(stats_a.likes, 1);
    assert_eq!(stats_a.dislikes, 0, "Les stats de A ne doivent pas inclure les dislikes de B");

    assert_eq!(stats_b.likes, 0, "Les stats de B ne doivent pas inclure les likes de A");
    assert_eq!(stats_b.dislikes, 1);
}

