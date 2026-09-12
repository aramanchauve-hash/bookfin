use leptos::prelude::*;
use uuid::Uuid;

use crate::application::dtos::{ExtractCardDto, ReactionResponseDto, ReactionTypeDto};

pub const FIXED_TEST_USER_ID: &str = "00000000-0000-0000-0000-000000000001";

#[server(GetNextExtract, "/api")]
pub async fn get_next_extract() -> Result<Option<ExtractCardDto>, ServerFnError> {
    use crate::application::use_cases::GetNextExtractUseCase;
    use crate::infrastructure::repositories::{
        PostgresExtractRepository, PostgresImpressionRepository,
    };
    use rand::Rng;
    use sqlx::PgPool;

    let pool = leptos_axum::extract::<axum::extract::Extension<PgPool>>()
        .await?
        .0;

    let user_id = Uuid::parse_str(FIXED_TEST_USER_ID)?;

    let rnd: f64 = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0)
    };

    let extract_repo = PostgresExtractRepository::new(pool.clone());
    let impression_repo = PostgresImpressionRepository::new(pool);
    let use_case = GetNextExtractUseCase::new(extract_repo, impression_repo);

    let result = match use_case.execute(user_id, rnd).await {
        Ok(res) => res,
        Err(e) => return Err(ServerFnError::new(e)),
    };

    Ok(result)
}

#[server(RecordReaction, "/api")]
pub async fn record_reaction(
    event_id: Uuid,
    extract_id: Uuid,
    reaction_type: ReactionTypeDto,
) -> Result<ReactionResponseDto, ServerFnError> {
    use crate::application::use_cases::RecordReactionUseCase;
    use crate::infrastructure::repositories::{
        PostgresExtractRepository, PostgresReactionRepository,
    };
    use sqlx::PgPool;

    let pool = leptos_axum::extract::<axum::extract::Extension<PgPool>>()
        .await?
        .0;

    let user_id = Uuid::parse_str(FIXED_TEST_USER_ID)?;

    let reaction_repo = PostgresReactionRepository::new(pool.clone());
    let extract_repo = PostgresExtractRepository::new(pool);
    let use_case = RecordReactionUseCase::new(reaction_repo, extract_repo);

    let result = match use_case
        .execute(event_id, user_id, extract_id, reaction_type)
        .await
    {
        Ok(res) => res,
        Err(e) => return Err(ServerFnError::new(e)),
    };

    Ok(result)
}
