use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use bookfin::application::ports::{ImpressionRepository, PageRepository, ReactionRepository};
use bookfin::application::use_cases::ContinueReadingUseCase;
use bookfin::domain::models::{NavigationAction, Page, Reaction, ReactionType};
use bookfin::infrastructure::db::init_db_pool;
use bookfin::infrastructure::repositories::{
    PostgresExtractRepository, PostgresImpressionRepository, PostgresPageRepository,
    PostgresReactionRepository,
};

async fn get_test_pool() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());
    init_db_pool(&url).await.ok()
}

#[tokio::test]
async fn test_reaction_persisted_and_idempotent() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };

    // Assurer les tables
    let res = sqlx::migrate!("./migrations").run(&pool).await;
    eprintln!("Migrate result: {:?}", res);

    // Créer un utilisateur et un extrait de test
    let user_id = Uuid::new_v4();
    let book_id = Uuid::new_v4();
    let extract_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Insertion user");

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Test Book")
    .bind("Test Author")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insertion work");

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Test Book")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insertion edition");

    let content1 = "Extrait de test d'intégration";
    let hash1 = Page::compute_hash(content1);
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, $3, $4, $5, $6, $7) ON CONFLICT (id) DO NOTHING",
    )
    .bind(extract_id)
    .bind(book_id)
    .bind(content1)
    .bind(hash1)
    .bind("fr")
    .bind(5)
    .bind(0.42)
    .execute(&pool)
    .await
    .expect("Insertion page");

    sqlx::query(
        "INSERT INTO books (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Test Book")
    .bind("Test Author")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insertion book");

    sqlx::query(
        "INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING",
    )
    .bind(extract_id)
    .bind(book_id)
    .bind("Extrait de test d'intégration")
    .bind("fr")
    .bind(5)
    .bind(0.42)
    .execute(&pool)
    .await
    .expect("Insertion extract");

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let event_id = Uuid::new_v4();

    let now = Utc::now();
    let reaction = Reaction {
        id: Uuid::new_v4(),
        event_id,
        user_id,
        page_id: extract_id,
        impression_id: None,
        reaction_type: ReactionType::Like,
        reading_time_ms: 5000,
        scroll_depth: 0.85,
        bottom_reached: true,
        content_overflows: false,
        navigation_action: Some(NavigationAction::ContinueBook),
        served_at: now,
        reacted_at: now,
        created_at: now,
        app_version: None,
        build_version: None,
    };

    // 1. Première écriture : doit être persistée avec succès
    let inserted = reaction_repo
        .record_reaction(&reaction)
        .await
        .expect("Enregistrement réussi");
    assert!(inserted, "La réaction doit être insérée");

    // Vérification en base
    let fetched = reaction_repo
        .get_reaction_by_event_id(event_id)
        .await
        .expect("Lecture réussie")
        .expect("Réaction trouvée");
    assert_eq!(fetched.event_id, event_id);
    assert_eq!(fetched.reaction_type, ReactionType::Like);

    // 2. Deuxième écriture avec le MÊME event_id (idempotence) : ne doit PAS créer de doublon
    let replayed = reaction_repo
        .record_reaction(&reaction)
        .await
        .expect("Requête rejouée sans erreur");
    assert!(
        !replayed,
        "La réaction rejouée ne doit pas insérer une nouvelle ligne"
    );

    // Vérifier qu'il n'y a toujours qu'une seule ligne
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reactions WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .expect("Count query");
    assert_eq!(
        count.0, 1,
        "Il ne doit y avoir qu'une seule réaction pour cet event_id"
    );
}

#[tokio::test]
async fn test_seen_extract_excluded_from_strategy() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let book_id = Uuid::new_v4();
    let extract_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Insertion user");

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Book Seen Test")
    .bind("Author Seen Test")
    .bind("en")
    .execute(&pool)
    .await
    .expect("Insertion work");

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Book Seen Test")
    .bind("en")
    .execute(&pool)
    .await
    .expect("Insertion edition");

    let seen_content = "Content for seen test";
    let seen_hash = Page::compute_hash(seen_content);
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, $3, $4, $5, $6, $7) ON CONFLICT (id) DO NOTHING",
    )
    .bind(extract_id)
    .bind(book_id)
    .bind(seen_content)
    .bind(seen_hash)
    .bind("en")
    .bind(4)
    .bind(0.123)
    .execute(&pool)
    .await
    .expect("Insertion page");

    sqlx::query(
        "INSERT INTO books (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Book Seen Test")
    .bind("Author Seen Test")
    .bind("en")
    .execute(&pool)
    .await
    .expect("Insertion book");

    sqlx::query(
        "INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING",
    )
    .bind(extract_id)
    .bind(book_id)
    .bind("Content for seen test")
    .bind("en")
    .bind(4)
    .bind(0.123)
    .execute(&pool)
    .await
    .expect("Insertion extract");

    let extract_repo = PostgresExtractRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool.clone());

    // Avant l'impression : l'extrait peut être sélectionné
    let candidate = extract_repo
        .get_unseen_random(user_id, 0.0)
        .await
        .expect("Requête avant impression");
    assert!(candidate.is_some());

    // Enregistrer l'impression (marqué comme vu)
    impression_repo
        .record_impression(user_id, extract_id, None)
        .await
        .expect("Enregistrement impression");

    assert!(
        impression_repo
            .has_impression(user_id, extract_id)
            .await
            .unwrap(),
        "L'impression doit être présente"
    );

    // Supprimer les autres extraits qui pourraient traîner pour cet utilisateur spécifique
    // Maintenant, extract_id ne doit plus être renvoyé si c'est le seul extrait disponible
    let unseen = extract_repo
        .get_unseen_random(user_id, 0.0)
        .await
        .expect("Requête après impression");
    if let Some(ext) = unseen {
        assert_ne!(
            ext.id, extract_id,
            "L'extrait déjà vu doit impérativement être exclu de la sélection"
        );
    }
}

#[tokio::test]
async fn test_continue_reading_sequential_pages() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();
    let page1_id = Uuid::new_v4();
    let page2_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Insertion user");

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(work_id)
    .bind("Roman Continu")
    .bind("Auteur Continu")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insertion work");

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, $3, $4)",
    )
    .bind(edition_id)
    .bind(work_id)
    .bind("Roman Continu")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insertion edition");

    // Page 1
    let content_p1 = "Voici le début de l'histoire qui commence sur la page un.";
    let hash_p1 = Page::compute_hash(content_p1);
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, source_page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, '1', $3, $4, 'fr', 50, 0.1)",
    )
    .bind(page1_id)
    .bind(edition_id)
    .bind(content_p1)
    .bind(hash_p1)
    .execute(&pool)
    .await
    .expect("Insertion page 1");

    // Page 2
    let content_p2 = "Et voici la suite immédiate de l'histoire sur la page deux.";
    let hash_p2 = Page::compute_hash(content_p2);
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, source_page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 2, '2', $3, $4, 'fr', 50, 0.2)",
    )
    .bind(page2_id)
    .bind(edition_id)
    .bind(content_p2)
    .bind(hash_p2)
    .execute(&pool)
    .await
    .expect("Insertion page 2");

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool.clone());

    // Vérification du port de pagination séquentielle
    let next_page = page_repo
        .get_next_page_in_edition(edition_id, 1)
        .await
        .expect("Query next page")
        .expect("Page 2 doit exister");
    assert_eq!(next_page.id, page2_id);
    assert_eq!(next_page.page_sequence_number(), 2);
    assert_eq!(next_page.source_page_number.as_deref(), Some("2"));

    // Page suivante après la dernière -> None
    let beyond_page = page_repo
        .get_next_page_in_edition(edition_id, 2)
        .await
        .expect("Query next page after end");
    assert!(beyond_page.is_none(), "Il n'y a pas de page 3");

    // Test du cas d'utilisation ContinueReadingUseCase
    let continue_use_case = ContinueReadingUseCase::new(page_repo, impression_repo.clone());
    let response = continue_use_case
        .execute(user_id, page1_id, None, None, None, None)
        .await
        .expect("Continue reading execute");

    assert_eq!(response.page_id, page2_id);
    assert_eq!(response.page_sequence_number, 2);
    assert_eq!(response.continuation_depth, 1);
    assert_eq!(response.source_page_number.as_deref(), Some("2"));
    assert_eq!(
        response.text,
        "Et voici la suite immédiate de l'histoire sur la page deux."
    );
}
