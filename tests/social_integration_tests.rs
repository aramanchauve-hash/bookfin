use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use bookfin::application::ports::{AffinityRepository, ConnectionRepository, ReactionRepository};
use bookfin::domain::models::{Page, Reaction, ReactionType};
use bookfin::infrastructure::db::init_db_pool;
use bookfin::infrastructure::repositories::{
    PostgresAffinityRepository, PostgresConnectionRepository, PostgresReactionRepository,
};

async fn get_test_pool() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());
    init_db_pool(&url).await.ok()
}

#[tokio::test]
async fn test_social_reaction_persisted_with_metrics() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("PostgreSQL inaccessible, test ignoré.");
        return;
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    let book_id = Uuid::new_v4();
    let extract_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("Insert user");

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Social Test Book")
    .bind("Author")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insert work");

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Social Test Book")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insert edition");

    let content = "Extrait de test pour les métriques sociales";
    let hash = Page::compute_hash(content);
    sqlx::query(
        "INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, 1, $3, $4, $5, $6, $7) ON CONFLICT (id) DO NOTHING",
    )
    .bind(extract_id)
    .bind(book_id)
    .bind(content)
    .bind(hash)
    .bind("fr")
    .bind(150)
    .bind(0.77)
    .execute(&pool)
    .await
    .expect("Insert page");

    sqlx::query(
        "INSERT INTO books (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Social Test Book")
    .bind("Author")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insert book");

    sqlx::query("INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING")
        .bind(extract_id)
        .bind(book_id)
        .bind("Extrait de test pour les métriques sociales")
        .bind("fr")
        .bind(150)
        .bind(0.77)
        .execute(&pool)
        .await
        .expect("Insert extract");

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let now = Utc::now();

    let reaction = Reaction {
        id: Uuid::new_v4(),
        event_id,
        user_id,
        page_id: extract_id,
        impression_id: None,
        reaction_type: ReactionType::Dislike,
        reading_time_ms: 6200,
        scroll_depth: 0.92,
        bottom_reached: true,
        content_overflows: false,
        navigation_action: None,
        served_at: now,
        reacted_at: now,
        created_at: now,
        app_version: None,
        build_version: None,
    };

    let inserted = reaction_repo
        .record_reaction(&reaction)
        .await
        .expect("Record reaction");
    assert!(inserted, "Réaction insérée");

    let fetched = reaction_repo
        .get_reaction_by_event_id(event_id)
        .await
        .expect("Query reaction")
        .expect("Found reaction");

    assert_eq!(fetched.reaction_type, ReactionType::Dislike);
    assert_eq!(fetched.reading_time_ms, 6200);
    assert!((fetched.scroll_depth - 0.92).abs() < 1e-4);
}

#[tokio::test]
async fn test_social_affinity_calculation_postgres() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("PostgreSQL inaccessible, test ignoré.");
        return;
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();
    let book_id = Uuid::new_v4();

    for u in &[user_a, user_b] {
        sqlx::query("INSERT INTO users (id) VALUES ($1)")
            .bind(u)
            .execute(&pool)
            .await
            .expect("Insert user");
    }

    sqlx::query(
        "INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Affinity Test Book")
    .bind("Author")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insert work");

    sqlx::query(
        "INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Affinity Test Book")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insert edition");

    sqlx::query(
        "INSERT INTO books (id, title, author, original_language_tag) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(book_id)
    .bind("Affinity Test Book")
    .bind("Author")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("Insert book");

    // Créer 4 pages et extraits
    let mut extract_ids = Vec::new();
    for i in 0..4 {
        let ext_id = Uuid::new_v4();
        let p_content = format!("Contenu de la page {}", i);
        let p_hash = Page::compute_hash(&p_content);
        sqlx::query("INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key) VALUES ($1, $2, $3, $4, $5, 'fr', 100, $6) ON CONFLICT (id) DO NOTHING")
            .bind(ext_id)
            .bind(book_id)
            .bind(i + 1)
            .bind(&p_content)
            .bind(p_hash)
            .bind(0.1 * (i as f64))
            .execute(&pool)
            .await
            .expect("Insert page");

        sqlx::query("INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING")
            .bind(ext_id)
            .bind(book_id)
            .bind(format!("Contenu de l'extrait {}", i))
            .bind("fr")
            .bind(100)
            .bind(0.1 * (i as f64))
            .execute(&pool)
            .await
            .expect("Insert extract");
        extract_ids.push(ext_id);
    }

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let now = Utc::now();

    // Configuration des réactions :
    // Extrait 0 : User A = like,    User B = like    -> commun like
    // Extrait 1 : User A = like,    User B = like    -> commun like
    // Extrait 2 : User A = dislike, User B = dislike -> commun dislike
    // Extrait 3 : User A = like,    User B = dislike -> désaccord
    let reactions_a = [
        (extract_ids[0], ReactionType::Like),
        (extract_ids[1], ReactionType::Like),
        (extract_ids[2], ReactionType::Dislike),
        (extract_ids[3], ReactionType::Like),
    ];
    let reactions_b = [
        (extract_ids[0], ReactionType::Like),
        (extract_ids[1], ReactionType::Like),
        (extract_ids[2], ReactionType::Dislike),
        (extract_ids[3], ReactionType::Dislike),
    ];

    for (ext_id, rtype) in reactions_a {
        reaction_repo
            .record_reaction(&Reaction {
                id: Uuid::new_v4(),
                event_id: Uuid::new_v4(),
                user_id: user_a,
                page_id: ext_id,
                impression_id: None,
                reaction_type: rtype,
                reading_time_ms: 5000,
                scroll_depth: 0.85,
                bottom_reached: true,
                content_overflows: false,
                navigation_action: None,
                served_at: now,
                reacted_at: now,
                created_at: now,
                app_version: None,
                build_version: None,
            })
            .await
            .expect("Insert reaction A");
    }

    for (ext_id, rtype) in reactions_b {
        reaction_repo
            .record_reaction(&Reaction {
                id: Uuid::new_v4(),
                event_id: Uuid::new_v4(),
                user_id: user_b,
                page_id: ext_id,
                impression_id: None,
                reaction_type: rtype,
                reading_time_ms: 5000,
                scroll_depth: 0.85,
                bottom_reached: true,
                content_overflows: false,
                navigation_action: None,
                served_at: now,
                reacted_at: now,
                created_at: now,
                app_version: None,
                build_version: None,
            })
            .await
            .expect("Insert reaction B");
    }

    let affinity_repo = PostgresAffinityRepository::new(pool.clone());
    let affinities = affinity_repo
        .calculate_user_affinities(
            user_a,
            1,
            None,
            &bookfin::domain::models::ReadingValidationConfig::default(),
        )
        .await
        .expect("Compute affinities");

    assert_eq!(affinities.len(), 1, "Un utilisateur cible trouvé");
    let aff = &affinities[0];
    assert_eq!(aff.target_user_id, user_b);
    assert_eq!(aff.common_likes, 2, "2 likes communs");
    assert_eq!(aff.common_dislikes, 1, "1 dislike commun");
    assert_eq!(aff.disagreements, 1, "1 désaccord");
    assert_eq!(aff.comparable_volume, 4, "4 lectures comparables au total");

    // Vérifier la persistance dans user_pair_affinities
    let (u1, u2) = if user_a < user_b {
        (user_a, user_b)
    } else {
        (user_b, user_a)
    };
    let row: (i32, i32, i32, i32) = sqlx::query_as(
        "SELECT common_likes, common_dislikes, disagreements, comparable_volume FROM user_pair_affinities WHERE user_a_id = $1 AND user_b_id = $2"
    )
    .bind(u1)
    .bind(u2)
    .fetch_one(&pool)
    .await
    .expect("Row persisted in user_pair_affinities");

    assert_eq!(row.0, 2);
    assert_eq!(row.1, 1);
    assert_eq!(row.2, 1);
    assert_eq!(row.3, 4);
}

#[tokio::test]
async fn test_connection_lifecycle_and_mutual_acceptance() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("PostgreSQL inaccessible, test ignoré.");
        return;
    };

    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let requester = Uuid::new_v4();
    let recipient = Uuid::new_v4();

    for u in &[requester, recipient] {
        sqlx::query("INSERT INTO users (id) VALUES ($1)")
            .bind(u)
            .execute(&pool)
            .await
            .expect("Insert user");
    }

    let connection_repo = PostgresConnectionRepository::new(pool.clone());

    // 1. Initialement, aucune proposition -> discussion impossible
    let status_none = connection_repo
        .get_connection_status(requester, recipient)
        .await
        .expect("Get initial status");
    assert!(status_none.is_none());

    // 2. Requester envoie une demande
    let proposal = connection_repo
        .request_connection(requester, recipient)
        .await
        .expect("Request connection");
    assert_eq!(proposal.status.as_str(), "pending");
    assert!(
        !proposal.can_start_discussion(),
        "Non ouvert tant que pending"
    );

    // Vérification du statut récupéré
    let fetched_pending = connection_repo
        .get_connection_status(requester, recipient)
        .await
        .expect("Query pending")
        .expect("Proposal exists");
    assert!(!fetched_pending.can_start_discussion());

    // 3. Recipient accepte la demande
    let accepted = connection_repo
        .accept_connection(requester, recipient)
        .await
        .expect("Accept connection");
    assert_eq!(accepted.status.as_str(), "accepted");
    assert!(
        accepted.can_start_discussion(),
        "Discussion autorisée après acceptation mutuelle"
    );

    // 4. Vérification symétrique (user_a, user_b) ou (user_b, user_a)
    let fetched_accepted = connection_repo
        .get_connection_status(recipient, requester)
        .await
        .expect("Query accepted sym")
        .expect("Proposal exists");
    assert!(fetched_accepted.can_start_discussion());
}
