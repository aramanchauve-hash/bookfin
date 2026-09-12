use sqlx::PgPool;
use uuid::Uuid;

use bookfin::application::dtos::ClaimAlphaInviteRequestDto;
use bookfin::application::ports::ImpressionRepository;
use bookfin::application::use_cases::{
    ClaimAlphaInviteUseCase, FeedNextPageUseCase, VerifyAlphaUserUseCase,
};
use bookfin::domain::errors::DomainError;
use bookfin::infrastructure::db::init_db_pool;
use bookfin::infrastructure::repositories::{
    PostgresAlphaInviteRepository, PostgresImpressionRepository, PostgresPageRepository,
};

async fn get_test_pool() -> Option<PgPool> {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/bookfin".to_string());
    init_db_pool(&url).await.ok()
}

#[tokio::test]
async fn test_alpha_invite_lifecycle_and_security() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let invite_repo = PostgresAlphaInviteRepository::new(pool.clone());
    let claim_use_case = ClaimAlphaInviteUseCase::new(invite_repo.clone());
    let verify_use_case = VerifyAlphaUserUseCase::new(invite_repo.clone());

    // 1. Code inexistant -> InvalidInviteCode
    let err_res = claim_use_case
        .execute(ClaimAlphaInviteRequestDto {
            code: "UNKNOWN-CODE-999".to_string(),
            device_summary: Some("Test Android".to_string()),
            app_version: Some("0.1.0-alpha".to_string()),
            build_version: Some("1".to_string()),
        })
        .await;
    assert!(
        matches!(err_res, Err(DomainError::InvalidInviteCode(_))),
        "Code inconnu doit être rejeté"
    );

    // 2. Création d'un code valide avec max_uses = 1
    let code_id = Uuid::new_v4();
    let test_code = format!("TEST-{}", &code_id.to_string()[..8]);
    sqlx::query(
        r#"
        INSERT INTO alpha_invite_codes (id, code, max_uses, uses_count, is_active, note)
        VALUES ($1, $2, 1, 0, true, 'Test auto')
        "#,
    )
    .bind(code_id)
    .bind(&test_code)
    .execute(&pool)
    .await
    .unwrap();

    // 3. Première réclamation réussie
    let claim_res = claim_use_case
        .execute(ClaimAlphaInviteRequestDto {
            code: test_code.clone(),
            device_summary: Some("Android SDK 34".to_string()),
            app_version: Some("0.1.0-alpha".to_string()),
            build_version: Some("1".to_string()),
        })
        .await
        .expect("Réclamation réussie");

    assert!(claim_res.claimed);
    let created_user_id = claim_res.user_id;

    // 4. Vérification de l'utilisateur créé
    let is_valid = verify_use_case.execute(created_user_id).await.unwrap();
    assert!(is_valid, "L'utilisateur alpha doit être reconnu comme valide");

    let is_fake_valid = verify_use_case.execute(Uuid::new_v4()).await.unwrap();
    assert!(!is_fake_valid, "Un UUID aléatoire inconnu ne doit pas être valide");

    // 5. Tentative de réutilisation du code (max_uses = 1 atteint) -> InviteCodeExpired
    let replay_res = claim_use_case
        .execute(ClaimAlphaInviteRequestDto {
            code: test_code.clone(),
            device_summary: Some("Android SDK 34".to_string()),
            app_version: Some("0.1.0-alpha".to_string()),
            build_version: Some("1".to_string()),
        })
        .await;
    assert!(
        matches!(replay_res, Err(DomainError::InviteCodeExpired(_))),
        "Code épuisé doit être rejeté"
    );
}

#[tokio::test]
async fn test_alpha_version_tracking_on_impressions() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Base de données non accessible, test ignoré.");
        return;
    };
    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id) VALUES ($1)").bind(user_id).execute(&pool).await.unwrap();

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool.clone());
    let feed_use_case = FeedNextPageUseCase::new(page_repo, impression_repo.clone());

    let feed_page = feed_use_case
        .execute(
            user_id,
            None,
            0.5,
            Some("0.1.0-alpha".to_string()),
            Some("42".to_string()),
        )
        .await
        .unwrap();

    if let Some(page) = feed_page {
        let stored_imp = impression_repo
            .get_impression_by_id(page.impression_id)
            .await
            .unwrap()
            .expect("Impression trouvée");

        assert_eq!(stored_imp.app_version.as_deref(), Some("0.1.0-alpha"));
        assert_eq!(stored_imp.build_version.as_deref(), Some("42"));
    }
}
