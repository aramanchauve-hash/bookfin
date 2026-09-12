use axum::{
    extract::{Extension, Path, Query},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::dtos::{
    ClaimAlphaInviteRequestDto, ClaimAlphaInviteResponseDto, ConnectionAcceptDto,
    ConnectionProposalDto, ConnectionRequestDto, ContinueReadingRequestDto, FeedPageDto,
    PageRevealDto, ReadingStatsDto, SubmitReactionRequestDto, SubmitReactionResponseDto,
    UserAffinityDto,
};
use crate::application::use_cases::{
    AcceptConnectionUseCase, CalculateAffinitiesUseCase, CanStartDiscussionUseCase,
    ClaimAlphaInviteUseCase, ContinueReadingUseCase, FeedNextPageUseCase, GetReadingStatsUseCase,
    RequestConnectionUseCase, RevealPageMetadataUseCase, SubmitReactionUseCase,
    VerifyAlphaUserUseCase,
};
use crate::domain::errors::DomainError;
use crate::domain::models::ReadingValidationConfig;
use crate::infrastructure::repositories::{
    PostgresAffinityRepository, PostgresAlphaInviteRepository, PostgresConnectionRepository,
    PostgresImpressionRepository, PostgresPageRepository, PostgresReactionRepository,
};

pub const DEFAULT_DEV_USER_ID: &str = "11111111-1111-1111-1111-111111111111";

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub user_id: Option<Uuid>,
    pub continue_from: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RevealQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AlphaVerifyQuery {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AffinitiesQuery {
    pub user_id: Option<Uuid>,
    pub min_overlap: Option<i32>,
    pub min_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionStatusQuery {
    pub current_user_id: Option<Uuid>,
}

pub fn create_v1_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/feed/next", get(feed_next_handler))
        .route("/pages/:page_id/continue", post(continue_reading_handler))
        .route("/reactions", post(reactions_handler))
        .route("/pages/:page_id/reveal", get(reveal_page_handler))
        .route("/alpha/claim", post(alpha_claim_handler))
        .route("/alpha/verify", get(alpha_verify_handler))
        .route("/me/stats", get(stats_handler))
        .route("/affinities", get(affinities_handler))
        .route("/connections/suggestions", get(suggestions_handler))
        .route(
            "/connections/:user_id/request",
            post(connection_request_handler),
        )
        .route(
            "/connections/:user_id/accept",
            post(connection_accept_handler),
        )
        .route(
            "/connections/:user_id/status",
            get(connection_status_handler),
        )
}

async fn continue_reading_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Path(page_id): Path<Uuid>,
    Json(payload): Json<Option<ContinueReadingRequestDto>>,
) -> Result<Json<FeedPageDto>, (StatusCode, String)> {
    let header_app_version = headers
        .get("X-App-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let header_build_version = headers
        .get("X-Build-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_id = payload
        .as_ref()
        .map(|p| p.user_id)
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());
    let parent_impression_id = payload.as_ref().and_then(|p| p.parent_impression_id);
    let session_id = payload.as_ref().and_then(|p| p.session_id);
    let app_version = payload
        .as_ref()
        .and_then(|p| p.app_version.clone())
        .or(header_app_version);
    let build_version = payload
        .as_ref()
        .and_then(|p| p.build_version.clone())
        .or(header_build_version);

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool);
    let use_case = ContinueReadingUseCase::new(page_repo, impression_repo);

    match use_case
        .execute(
            user_id,
            page_id,
            parent_impression_id,
            session_id,
            app_version,
            build_version,
        )
        .await
    {
        Ok(dto) => Ok(Json(dto)),
        Err(DomainError::EndOfEdition {
            edition_id,
            last_page_number,
        }) => Err((
            StatusCode::NOT_FOUND,
            serde_json::json!({
                "error": "end_of_edition",
                "edition_id": edition_id,
                "last_page_number": last_page_number,
                "message": "Fin du livre atteinte : aucune page suivante disponible dans cette édition"
            })
            .to_string(),
        )),
        Err(DomainError::PageNotFound(_)) => {
            Err((StatusCode::NOT_FOUND, "Page introuvable".to_string()))
        }
        Err(DomainError::InvalidContinuation(msg)) => Err((StatusCode::BAD_REQUEST, msg)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn feed_next_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Query(query): Query<FeedQuery>,
) -> Result<Json<FeedPageDto>, (StatusCode, String)> {
    let header_app_version = headers
        .get("X-App-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let header_build_version = headers
        .get("X-Build-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_id = query
        .user_id
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());

    let page_repo = PostgresPageRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool);

    if let Some(continue_from_id) = query.continue_from {
        let use_case = ContinueReadingUseCase::new(page_repo, impression_repo);
        match use_case
            .execute(
                user_id,
                continue_from_id,
                None,
                None,
                header_app_version,
                header_build_version,
            )
            .await
        {
            Ok(dto) => return Ok(Json(dto)),
            Err(DomainError::EndOfEdition {
                edition_id,
                last_page_number,
            }) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    serde_json::json!({
                        "error": "end_of_edition",
                        "edition_id": edition_id,
                        "last_page_number": last_page_number,
                        "message": "Fin du livre atteinte : aucune page suivante disponible dans cette édition"
                    })
                    .to_string(),
                ))
            }
            Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        }
    }

    let use_case = FeedNextPageUseCase::new(page_repo, impression_repo);

    let rnd = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0)
    };

    match use_case
        .execute(user_id, None, rnd, header_app_version, header_build_version)
        .await
    {
        Ok(Some(dto)) => Ok(Json(dto)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            "Aucune nouvelle page disponible dans la base".to_string(),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn reactions_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(mut payload): Json<SubmitReactionRequestDto>,
) -> Result<(StatusCode, Json<SubmitReactionResponseDto>), (StatusCode, String)> {
    let header_app_version = headers
        .get("X-App-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let header_build_version = headers
        .get("X-Build-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if payload.app_version.is_none() {
        payload.app_version = header_app_version;
    }
    if payload.build_version.is_none() {
        payload.build_version = header_build_version;
    }

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool);
    let use_case = SubmitReactionUseCase::new(reaction_repo, impression_repo);
    let config = ReadingValidationConfig::default();

    match use_case.execute(payload, &config).await {
        Ok(resp) => Ok((StatusCode::CREATED, Json(resp))),
        Err(DomainError::ReadingTooFast { .. })
        | Err(DomainError::InsufficientScroll { .. })
        | Err(DomainError::ServerTimeElapsedTooShort { .. }) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Metriques de lecture invalides ou delai insuffisant".to_string(),
        )),
        Err(DomainError::DuplicateReaction) | Err(DomainError::AlreadyReacted) => Err((
            StatusCode::CONFLICT,
            "Cette impression a déjà reçu une réaction".to_string(),
        )),
        Err(DomainError::ImpressionNotFound(msg)) => Err((
            StatusCode::NOT_FOUND,
            format!("Impression introuvable : {}", msg),
        )),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn alpha_claim_handler(
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
    Json(mut payload): Json<ClaimAlphaInviteRequestDto>,
) -> Result<(StatusCode, Json<ClaimAlphaInviteResponseDto>), (StatusCode, String)> {
    let header_app_version = headers
        .get("X-App-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let header_build_version = headers
        .get("X-Build-Version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if payload.app_version.is_none() {
        payload.app_version = header_app_version;
    }
    if payload.build_version.is_none() {
        payload.build_version = header_build_version;
    }

    let invite_repo = PostgresAlphaInviteRepository::new(pool);
    let use_case = ClaimAlphaInviteUseCase::new(invite_repo);

    match use_case.execute(payload).await {
        Ok(resp) => Ok((StatusCode::CREATED, Json(resp))),
        Err(DomainError::InvalidInviteCode(msg)) => Err((
            StatusCode::FORBIDDEN,
            serde_json::json!({ "error": "invalid_code", "message": msg }).to_string(),
        )),
        Err(DomainError::InviteCodeExpired(msg)) => Err((
            StatusCode::GONE,
            serde_json::json!({ "error": "code_expired", "message": msg }).to_string(),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn alpha_verify_handler(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<AlphaVerifyQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let invite_repo = PostgresAlphaInviteRepository::new(pool);
    let use_case = VerifyAlphaUserUseCase::new(invite_repo);

    match use_case.execute(query.user_id).await {
        Ok(valid) => Ok(Json(serde_json::json!({
            "valid": valid,
            "user_id": query.user_id
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn reveal_page_handler(
    Extension(pool): Extension<PgPool>,
    Path(page_id): Path<Uuid>,
    Query(query): Query<RevealQuery>,
) -> Result<Json<PageRevealDto>, (StatusCode, String)> {
    let user_id = query
        .user_id
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());

    let page_repo = PostgresPageRepository::new(pool.clone());
    let reaction_repo = PostgresReactionRepository::new(pool);
    let use_case = RevealPageMetadataUseCase::new(page_repo, reaction_repo);

    match use_case.execute(user_id, page_id).await {
        Ok(dto) => Ok(Json(dto)),
        Err(DomainError::ReactionRequiredForReveal) => Err((
            StatusCode::FORBIDDEN,
            "Une réaction préalable est obligatoire pour révéler les métadonnées de cette page"
                .to_string(),
        )),
        Err(DomainError::PageNotFound(_)) => {
            Err((StatusCode::NOT_FOUND, "Page introuvable".to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn stats_handler(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<ReadingStatsDto>, (StatusCode, String)> {
    let user_id = query
        .user_id
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());

    let reaction_repo = PostgresReactionRepository::new(pool);
    let use_case = GetReadingStatsUseCase::new(reaction_repo);

    match use_case.execute(user_id).await {
        Ok(dto) => Ok(Json(dto)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn affinities_handler(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<AffinitiesQuery>,
) -> Result<Json<Vec<UserAffinityDto>>, (StatusCode, String)> {
    let user_id = query
        .user_id
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());

    let affinity_repo = PostgresAffinityRepository::new(pool);
    let use_case = CalculateAffinitiesUseCase::new(affinity_repo);
    let config = ReadingValidationConfig::default();

    match use_case
        .execute(user_id, query.min_overlap.unwrap_or(1), query.min_score, &config)
        .await
    {
        Ok(dtos) => Ok(Json(dtos)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn suggestions_handler(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<AffinitiesQuery>,
) -> Result<Json<Vec<UserAffinityDto>>, (StatusCode, String)> {
    let user_id = query
        .user_id
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());

    let affinity_repo = PostgresAffinityRepository::new(pool);
    let use_case = CalculateAffinitiesUseCase::new(affinity_repo);
    let config = ReadingValidationConfig::default();

    match use_case
        .execute(user_id, query.min_overlap.unwrap_or(1), query.min_score, &config)
        .await
    {
        Ok(dtos) => {
            let filtered: Vec<UserAffinityDto> =
                dtos.into_iter().filter(|a| a.can_connect).collect();
            Ok(Json(filtered))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn connection_request_handler(
    Extension(pool): Extension<PgPool>,
    Path(recipient_id): Path<Uuid>,
    Json(payload): Json<ConnectionRequestDto>,
) -> Result<(StatusCode, Json<ConnectionProposalDto>), (StatusCode, String)> {
    let connection_repo = PostgresConnectionRepository::new(pool);
    let use_case = RequestConnectionUseCase::new(connection_repo);

    match use_case.execute(payload.requester_id, recipient_id).await {
        Ok(dto) => Ok((StatusCode::CREATED, Json(dto))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn connection_accept_handler(
    Extension(pool): Extension<PgPool>,
    Path(requester_id): Path<Uuid>,
    Json(payload): Json<ConnectionAcceptDto>,
) -> Result<Json<ConnectionProposalDto>, (StatusCode, String)> {
    let connection_repo = PostgresConnectionRepository::new(pool);
    let use_case = AcceptConnectionUseCase::new(connection_repo);

    match use_case.execute(requester_id, payload.recipient_id).await {
        Ok(dto) => Ok(Json(dto)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn connection_status_handler(
    Extension(pool): Extension<PgPool>,
    Path(other_user_id): Path<Uuid>,
    Query(query): Query<ConnectionStatusQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let current_user_id = query
        .current_user_id
        .unwrap_or_else(|| Uuid::parse_str(DEFAULT_DEV_USER_ID).unwrap());

    let connection_repo = PostgresConnectionRepository::new(pool);
    let use_case = CanStartDiscussionUseCase::new(connection_repo);

    match use_case.execute(current_user_id, other_user_id).await {
        Ok(can_start) => Ok(Json(serde_json::json!({
            "current_user_id": current_user_id,
            "other_user_id": other_user_id,
            "can_start_discussion": can_start,
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
