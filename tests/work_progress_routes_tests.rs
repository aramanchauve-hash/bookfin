//! Tests d'intégration HTTP de la lecture suivie et de la bibliothèque
//! (`/works/:work_id/start`, `/works/:work_id/resume`, `/me/library`) et de
//! l'invariant de séparation découverte / lecture suivie.
//!
//! Comme les autres tests d'intégration, ils utilisent la base locale
//! (`DATABASE_URL`) et sont ignorés si elle est inaccessible. Chaque test crée
//! ses propres lecteurs, œuvre, édition et pages (UUID aléatoires).

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Extension, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use bookfin::application::ports::ImpressionRepository;
use bookfin::infrastructure::db::init_db_pool;
use bookfin::infrastructure::repositories::PostgresImpressionRepository;
use bookfin::web::api_v1::create_v1_router;

const PAGE_COUNT: i32 = 110;

async fn get_test_pool() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());
    let pool = init_db_pool(&url).await.ok()?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations locales");
    Some(pool)
}

struct Fixture {
    pool: PgPool,
    app: Router,
    work_id: Uuid,
    edition_id: Uuid,
}

async fn fixture() -> Option<Fixture> {
    let pool = get_test_pool().await?;
    let work_id = Uuid::new_v4();
    let edition_id = Uuid::new_v4();

    sqlx::query("INSERT INTO works (id, title, author, original_language_tag) VALUES ($1, 'Niebla (test)', 'Unamuno (test)', 'es')")
        .bind(work_id)
        .execute(&pool)
        .await
        .expect("work");
    sqlx::query("INSERT INTO editions (id, work_id, edition_title, language_tag) VALUES ($1, $2, 'Édition test', 'es')")
        .bind(edition_id)
        .bind(work_id)
        .execute(&pool)
        .await
        .expect("edition");
    sqlx::query(
        r#"
        INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key)
        SELECT gen_random_uuid(), $1, n, c, encode(sha256(convert_to(c, 'UTF8')), 'hex'), 'es', 5, random() * 0.999
        FROM (SELECT n, 'Página ' || n || ' ' || $1::text AS c FROM generate_series(1, $2) AS n) s
        "#,
    )
    .bind(edition_id)
    .bind(PAGE_COUNT)
    .execute(&pool)
    .await
    .expect("pages");

    let app = create_v1_router::<()>().layer(Extension(pool.clone()));
    Some(Fixture {
        pool,
        app,
        work_id,
        edition_id,
    })
}

async fn new_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id) VALUES ($1)")
        .bind(id)
        .execute(pool)
        .await
        .expect("user");
    id
}

async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(v) => {
            builder = builder.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .expect("réponse");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, value)
}

async fn start(f: &Fixture, user: Uuid, restart: bool) -> (StatusCode, Value) {
    call(
        &f.app,
        "POST",
        &format!("/works/{}/start", f.work_id),
        Some(json!({ "user_id": user, "edition_id": f.edition_id, "restart": restart })),
    )
    .await
}

async fn resume(f: &Fixture, user: Uuid) -> (StatusCode, Value) {
    call(
        &f.app,
        "POST",
        &format!("/works/{}/resume", f.work_id),
        Some(json!({ "user_id": user })),
    )
    .await
}

async fn continue_from(f: &Fixture, user: Uuid, served: &Value) -> (StatusCode, Value) {
    call(
        &f.app,
        "POST",
        &format!("/pages/{}/continue", served["page_id"].as_str().unwrap()),
        Some(json!({ "user_id": user, "parent_impression_id": served["impression_id"] })),
    )
    .await
}

async fn progress_page(pool: &PgPool, user: Uuid, work: Uuid) -> Option<i32> {
    sqlx::query_scalar("SELECT current_page_sequence_number FROM user_work_progress WHERE user_id = $1 AND work_id = $2")
        .bind(user)
        .bind(work)
        .fetch_optional(pool)
        .await
        .unwrap()
}

async fn set_progress(pool: &PgPool, user: Uuid, f: &Fixture, page_number: i32) {
    sqlx::query(
        r#"
        UPDATE user_work_progress
        SET current_page_sequence_number = $3,
            current_page_id = (SELECT id FROM pages WHERE edition_id = $4 AND page_number = $3)
        WHERE user_id = $1 AND work_id = $2
        "#,
    )
    .bind(user)
    .bind(f.work_id)
    .bind(page_number)
    .bind(f.edition_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn page_id(pool: &PgPool, edition: Uuid, number: i32) -> Uuid {
    sqlx::query_scalar("SELECT id FROM pages WHERE edition_id = $1 AND page_number = $2")
        .bind(edition)
        .bind(number)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn impression_context(pool: &PgPool, impression_id: &Value) -> (String, Option<String>) {
    let id = Uuid::parse_str(impression_id.as_str().unwrap()).unwrap();
    sqlx::query_as("SELECT reading_context, navigation_action FROM page_impressions WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn count_events(pool: &PgPool, user: Uuid, event_type: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM reading_events WHERE user_id = $1 AND event_type = $2")
        .bind(user)
        .bind(event_type)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Impressions comptées comme RANDOM_PAGE_IMPRESSION par les analyses du feed.
async fn count_random_impressions(pool: &PgPool, user: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM page_impressions WHERE user_id = $1 AND reading_context = 'discovery' AND navigation_action = 'random_page'",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn start_valid_work_creates_progress_at_page_one_in_sequential_context() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;

    let (status, page) = start(&f, user, false).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!(page["page_sequence_number"], 1);
    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(1));
    assert_eq!(count_events(&f.pool, user, "start_work").await, 1);

    let (context, action) = impression_context(&f.pool, &page["impression_id"]).await;
    assert_eq!(context, "sequential");
    assert_eq!(action.as_deref(), Some("start_work"));
    assert_eq!(count_random_impressions(&f.pool, user).await, 0);
}

#[tokio::test]
async fn start_rejects_unknown_work_mismatched_edition_and_unknown_user() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;

    let (status, _) = call(
        &f.app,
        "POST",
        &format!("/works/{}/start", Uuid::new_v4()),
        Some(json!({ "user_id": user, "edition_id": f.edition_id })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "œuvre inexistante");

    let (status, _) = call(
        &f.app,
        "POST",
        &format!("/works/{}/start", f.work_id),
        Some(json!({ "user_id": user, "edition_id": Uuid::new_v4() })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "édition d'une autre œuvre");

    let (status, _) = start(&f, Uuid::new_v4(), false).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "identité inconnue");

    assert_eq!(progress_page(&f.pool, user, f.work_id).await, None);
}

#[tokio::test]
async fn start_on_already_started_work_is_refused_and_keeps_progress() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;
    assert_eq!(start(&f, user, false).await.0, StatusCode::OK);
    set_progress(&f.pool, user, &f, 47).await;

    let (status, payload) = start(&f, user, false).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(payload["error"], "work_already_started");
    assert_eq!(payload["current_page_sequence_number"], 47);

    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(47));
    assert_eq!(count_events(&f.pool, user, "start_work").await, 1, "pas de START_WORK dupliqué");
}

#[tokio::test]
async fn explicit_restart_resets_progress_to_page_one() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;
    assert_eq!(start(&f, user, false).await.0, StatusCode::OK);
    set_progress(&f.pool, user, &f, 47).await;

    let (status, page) = start(&f, user, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["page_sequence_number"], 1);
    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(1));
    assert_eq!(count_events(&f.pool, user, "restart_work").await, 1);
    assert_eq!(count_events(&f.pool, user, "start_work").await, 1);
}

#[tokio::test]
async fn resume_serves_saved_page_without_moving_progress() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;

    let (status, _) = resume(&f, user).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "aucune progression");

    assert_eq!(start(&f, user, false).await.0, StatusCode::OK);
    set_progress(&f.pool, user, &f, 47).await;

    let (status, page) = resume(&f, user).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["page_sequence_number"], 47);
    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(47));
    assert_eq!(count_events(&f.pool, user, "resume_work").await, 1);

    let (context, action) = impression_context(&f.pool, &page["impression_id"]).await;
    assert_eq!(context, "sequential");
    assert_eq!(action.as_deref(), Some("resume_work"));
}

#[tokio::test]
async fn sequential_continue_advances_progress_and_stays_sequential() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;

    let (_, page1) = start(&f, user, false).await;
    let (status, page2) = continue_from(&f, user, &page1).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page2["page_sequence_number"], 2);
    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(2));

    let (_, page3) = continue_from(&f, user, &page2).await;
    assert_eq!(page3["page_sequence_number"], 3);
    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(3));

    let (context, action) = impression_context(&f.pool, &page3["impression_id"]).await;
    assert_eq!(context, "sequential");
    assert_eq!(action.as_deref(), Some("continue_book"));
    assert_eq!(count_random_impressions(&f.pool, user).await, 0);
}

/// Régression P1 : progression(Niebla) = 47, le feed aléatoire sert Niebla p. 103,
/// le lecteur continue dans le feed → la progression reste 47.
#[tokio::test]
async fn random_feed_continue_never_overwrites_work_progress() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;
    assert_eq!(start(&f, user, false).await.0, StatusCode::OK);
    set_progress(&f.pool, user, &f, 47).await;

    // Impression de découverte identique à celle du feed aléatoire, sur la page 103.
    let random_page = page_id(&f.pool, f.edition_id, 103).await;
    let impression = PostgresImpressionRepository::new(f.pool.clone())
        .record_impression(user, random_page, None)
        .await
        .unwrap();
    let served = json!({ "page_id": random_page, "impression_id": impression.id });

    let (status, next) = continue_from(&f, user, &served).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(next["page_sequence_number"], 104);
    let (_, next2) = continue_from(&f, user, &next).await;
    assert_eq!(next2["page_sequence_number"], 105);

    assert_eq!(progress_page(&f.pool, user, f.work_id).await, Some(47));
    let (context, _) = impression_context(&f.pool, &next["impression_id"]).await;
    assert_eq!(context, "discovery");

    // La reprise sert toujours la page 47.
    let (_, resumed) = resume(&f, user).await;
    assert_eq!(resumed["page_sequence_number"], 47);
}

#[tokio::test]
async fn library_lists_started_works_only_and_isolates_users() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let reader = new_user(&f.pool).await;
    let other = new_user(&f.pool).await;

    let (status, empty) = call(&f.app, "GET", &format!("/me/library?user_id={reader}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty, json!([]));

    assert_eq!(start(&f, reader, false).await.0, StatusCode::OK);
    set_progress(&f.pool, reader, &f, 47).await;

    let (_, items) = call(&f.app, "GET", &format!("/me/library?user_id={reader}"), None).await;
    let items = items.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["work_id"], json!(f.work_id));
    assert_eq!(items[0]["edition_id"], json!(f.edition_id));
    assert_eq!(items[0]["current_page_sequence_number"], 47);
    assert_eq!(items[0]["total_pages"], PAGE_COUNT);

    let (_, other_items) = call(&f.app, "GET", &format!("/me/library?user_id={other}"), None).await;
    assert_eq!(other_items, json!([]), "la bibliothèque d'un lecteur n'est pas visible par un autre");

    let (status, _) = resume(&f, other).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "aucune reprise de la progression d'un autre lecteur");
    assert_eq!(progress_page(&f.pool, reader, f.work_id).await, Some(47));
}

#[tokio::test]
async fn reveal_reports_existing_progress_only_when_work_started() {
    let Some(f) = fixture().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let user = new_user(&f.pool).await;
    let page = page_id(&f.pool, f.edition_id, 10).await;
    sqlx::query("INSERT INTO reactions (id, event_id, user_id, reaction_type, page_id) VALUES ($1, $2, $3, 'like', $4)")
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(user)
        .bind(page)
        .execute(&f.pool)
        .await
        .unwrap();

    let (status, meta) = call(&f.app, "GET", &format!("/pages/{page}/reveal?user_id={user}"), None).await;
    assert_eq!(status, StatusCode::OK, "{meta}");
    assert_eq!(meta["work_id"], json!(f.work_id));
    assert!(meta.get("work_progress_page_sequence_number").is_none());

    assert_eq!(start(&f, user, false).await.0, StatusCode::OK);
    set_progress(&f.pool, user, &f, 47).await;
    let (_, meta) = call(&f.app, "GET", &format!("/pages/{page}/reveal?user_id={user}"), None).await;
    assert_eq!(meta["work_progress_page_sequence_number"], 47);
}
