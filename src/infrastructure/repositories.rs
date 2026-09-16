#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc};
#[cfg(feature = "ssr")]
use sqlx::{types::Json, FromRow, PgPool};
#[cfg(feature = "ssr")]
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::application::ports::{
    AffinityRepository, AlphaInviteRepository, ConnectionRepository, ImpressionRepository,
    PageRepository, ReactionRepository,
};
#[cfg(feature = "ssr")]
use crate::domain::errors::DomainError;
#[cfg(feature = "ssr")]
use crate::domain::models::{
    AlphaInviteCode, ConnectionProposal, ConnectionStatus, LanguageTag, Page, PageContentV2, PageImpression,
    PageMetadata, Reaction, ReactionType, ReadingStats, ReadingValidationConfig, UserAffinity,
};

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PostgresPageRepository {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl PostgresPageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlPage {
    id: Uuid,
    edition_id: Uuid,
    page_number: i32,
    source_page_number: Option<String>,
    content: String,
    content_v2: Option<Json<PageContentV2>>,
    content_hash: String,
    language_tag: String,
    token_count: i32,
    random_key: f64,
    is_active: bool,
    version: i32,
    created_at: DateTime<Utc>,
}

#[cfg(feature = "ssr")]
impl SqlPage {
    fn into_domain(self) -> Result<Page, DomainError> {
        let language_tag = LanguageTag::parse(&self.language_tag)?;
        Ok(Page {
            id: self.id,
            edition_id: self.edition_id,
            page_number: self.page_number,
            source_page_number: self.source_page_number,
            content: self.content,
            content_v2: self.content_v2.map(|Json(content)| content),
            content_hash: self.content_hash,
            language_tag,
            token_count: self.token_count,
            random_key: self.random_key,
            is_active: self.is_active,
            version: self.version,
            created_at: self.created_at,
        })
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlPageMetadata {
    title: String,
    author: String,
    original_language_tag: String,
    edition_title: String,
    translator: Option<String>,
    publication_year: Option<i32>,
    source_name: String,
}

#[cfg(feature = "ssr")]
impl PageRepository for PostgresPageRepository {
    async fn get_unseen_random(
        &self,
        user_id: Uuid,
        rnd: f64,
    ) -> Result<Option<Page>, DomainError> {
        // 1. Essai après la clé aléatoire
        let candidate = sqlx::query_as::<_, SqlPage>(
            r#"
            SELECT id, edition_id, page_number, source_page_number, content, content_v2, content_hash, language_tag,
                   token_count, random_key, is_active, version, created_at
            FROM pages p
            WHERE p.is_active = true
              -- New readers always have preferences (set during onboarding).
              -- The first branch is a narrow compatibility bridge for existing
              -- alpha identities created before preferences existed; it avoids
              -- silently invalidating their historical sessions.
              AND (
                  NOT EXISTS (
                      SELECT 1 FROM user_language_preferences ulp WHERE ulp.user_id = $1
                  )
                  OR EXISTS (
                      SELECT 1 FROM user_language_preferences ulp
                      WHERE ulp.user_id = $1 AND ulp.language_tag = p.language_tag
                  )
              )
              AND NOT EXISTS (
                  SELECT 1 FROM page_impressions pi WHERE pi.user_id = $1 AND pi.page_id = p.id
              )
              AND p.random_key >= $2
            ORDER BY p.random_key ASC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(rnd)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_unseen_random: {}", e)))?;

        if let Some(row) = candidate {
            return row.into_domain().map(Some);
        }

        // 2. Wrap-around : recommence au début de l'index random_key
        let wrap_candidate = sqlx::query_as::<_, SqlPage>(
            r#"
            SELECT id, edition_id, page_number, source_page_number, content, content_v2, content_hash, language_tag,
                   token_count, random_key, is_active, version, created_at
            FROM pages p
            WHERE p.is_active = true
              AND (
                  NOT EXISTS (
                      SELECT 1 FROM user_language_preferences ulp WHERE ulp.user_id = $1
                  )
                  OR EXISTS (
                      SELECT 1 FROM user_language_preferences ulp
                      WHERE ulp.user_id = $1 AND ulp.language_tag = p.language_tag
                  )
              )
              AND NOT EXISTS (
                  SELECT 1 FROM page_impressions pi WHERE pi.user_id = $1 AND pi.page_id = p.id
              )
              ORDER BY p.random_key ASC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_unseen_random wrap-around: {}", e)))?;

        match wrap_candidate {
            Some(row) => row.into_domain().map(Some),
            None => Ok(None),
        }
    }

    async fn get_metadata_for_page(
        &self,
        page_id: Uuid,
    ) -> Result<Option<PageMetadata>, DomainError> {
        let row = sqlx::query_as::<_, SqlPageMetadata>(
            r#"
            SELECT w.title, w.author, w.original_language_tag,
                   e.edition_title, e.translator, e.publication_year, e.source_name
            FROM pages p
            JOIN editions e ON p.edition_id = e.id
            JOIN works w ON e.work_id = w.id
            WHERE p.id = $1
            "#,
        )
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_metadata_for_page: {}", e)))?;

        Ok(row.map(|r| PageMetadata {
            title: r.title,
            author: r.author,
            original_language_tag: r.original_language_tag,
            edition_title: r.edition_title,
            translator: r.translator,
            publication_year: r.publication_year,
            source_name: r.source_name,
        }))
    }

    async fn get_page_by_id(&self, page_id: Uuid) -> Result<Option<Page>, DomainError> {
        let row = sqlx::query_as::<_, SqlPage>(
            r#"
            SELECT id, edition_id, page_number, source_page_number, content, content_v2, content_hash, language_tag,
                   token_count, random_key, is_active, version, created_at
            FROM pages
            WHERE id = $1
            "#,
        )
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_page_by_id: {}", e)))?;

        match row {
            Some(r) => r.into_domain().map(Some),
            None => Ok(None),
        }
    }

    async fn get_next_page_in_edition(
        &self,
        edition_id: Uuid,
        current_page_number: i32,
    ) -> Result<Option<Page>, DomainError> {
        let row = sqlx::query_as::<_, SqlPage>(
            r#"
            SELECT id, edition_id, page_number, source_page_number, content, content_v2, content_hash, language_tag,
                   token_count, random_key, is_active, version, created_at
            FROM pages
            WHERE edition_id = $1
              AND page_number = $2 + 1
              AND is_active = true
            ORDER BY page_number ASC
            LIMIT 1
            "#,
        )
        .bind(edition_id)
        .bind(current_page_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_next_page_in_edition: {}", e)))?;

        match row {
            Some(r) => r.into_domain().map(Some),
            None => Ok(None),
        }
    }
}

pub type PostgresExtractRepository = PostgresPageRepository;

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PostgresImpressionRepository {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl PostgresImpressionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlPageImpression {
    id: Uuid,
    user_id: Uuid,
    page_id: Uuid,
    served_at: DateTime<Utc>,
    session_id: Option<Uuid>,
    parent_impression_id: Option<Uuid>,
    root_page_id: Option<Uuid>,
    continuation_depth: i32,
    active_reading_time_ms: Option<i32>,
    scroll_depth: Option<f64>,
    bottom_reached: bool,
    scroll_back: bool,
    navigation_action: Option<String>,
    book_resumed: bool,
    is_bookmarked: bool,
    reached_end: bool,
    app_version: Option<String>,
    build_version: Option<String>,
}

#[cfg(feature = "ssr")]
impl SqlPageImpression {
    fn into_domain(self) -> Result<PageImpression, DomainError> {
        let navigation_action = self
            .navigation_action
            .as_deref()
            .map(str::parse)
            .transpose()?;

        Ok(PageImpression {
            id: self.id,
            user_id: self.user_id,
            page_id: self.page_id,
            served_at: self.served_at,
            session_id: self.session_id,
            parent_impression_id: self.parent_impression_id,
            root_page_id: self.root_page_id,
            continuation_depth: self.continuation_depth,
            active_reading_time_ms: self.active_reading_time_ms,
            scroll_depth: self.scroll_depth,
            bottom_reached: self.bottom_reached,
            scroll_back: self.scroll_back,
            navigation_action,
            book_resumed: self.book_resumed,
            is_bookmarked: self.is_bookmarked,
            reached_end: self.reached_end,
            app_version: self.app_version,
            build_version: self.build_version,
        })
    }
}

#[cfg(feature = "ssr")]
impl ImpressionRepository for PostgresImpressionRepository {
    async fn record_impression(
        &self,
        user_id: Uuid,
        page_id: Uuid,
        session_id: Option<Uuid>,
    ) -> Result<PageImpression, DomainError> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, SqlPageImpression>(
            r#"
            INSERT INTO page_impressions (
                id, user_id, page_id, served_at, session_id,
                parent_impression_id, root_page_id, continuation_depth,
                active_reading_time_ms, scroll_depth, bottom_reached,
                scroll_back, navigation_action, book_resumed, is_bookmarked,
                reached_end, app_version, build_version
            )
            VALUES ($1, $2, $3, NOW(), $4, NULL, $3, 0, NULL, NULL, false, false, 'random_page', false, false, false, NULL, NULL)
            RETURNING id, user_id, page_id, served_at, session_id,
                      parent_impression_id, root_page_id, continuation_depth,
                      active_reading_time_ms, scroll_depth, bottom_reached,
                      scroll_back, navigation_action, book_resumed, is_bookmarked,
                      reached_end, app_version, build_version
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(page_id)
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL record_impression: {}", e)))?;

        row.into_domain()
    }

    async fn record_impression_full(
        &self,
        impression: &PageImpression,
    ) -> Result<PageImpression, DomainError> {
        let row = sqlx::query_as::<_, SqlPageImpression>(
            r#"
            INSERT INTO page_impressions (
                id, user_id, page_id, served_at, session_id,
                parent_impression_id, root_page_id, continuation_depth,
                active_reading_time_ms, scroll_depth, bottom_reached,
                scroll_back, navigation_action, book_resumed, is_bookmarked,
                reached_end, app_version, build_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING id, user_id, page_id, served_at, session_id,
                      parent_impression_id, root_page_id, continuation_depth,
                      active_reading_time_ms, scroll_depth, bottom_reached,
                      scroll_back, navigation_action, book_resumed, is_bookmarked,
                      reached_end, app_version, build_version
            "#,
        )
        .bind(impression.id)
        .bind(impression.user_id)
        .bind(impression.page_id)
        .bind(impression.served_at)
        .bind(impression.session_id)
        .bind(impression.parent_impression_id)
        .bind(impression.root_page_id)
        .bind(impression.continuation_depth)
        .bind(impression.active_reading_time_ms)
        .bind(impression.scroll_depth)
        .bind(impression.bottom_reached)
        .bind(impression.scroll_back)
        .bind(impression.navigation_action.map(|a| a.as_str().to_string()))
        .bind(impression.book_resumed)
        .bind(impression.is_bookmarked)
        .bind(impression.reached_end)
        .bind(impression.app_version.as_deref())
        .bind(impression.build_version.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL record_impression_full: {}", e)))?;

        row.into_domain()
    }

    async fn get_impression(
        &self,
        user_id: Uuid,
        page_id: Uuid,
    ) -> Result<Option<PageImpression>, DomainError> {
        let row = sqlx::query_as::<_, SqlPageImpression>(
            r#"
            SELECT id, user_id, page_id, served_at, session_id,
                   parent_impression_id, root_page_id, continuation_depth,
                   active_reading_time_ms, scroll_depth, bottom_reached,
                   scroll_back, navigation_action, book_resumed, is_bookmarked,
                   reached_end, app_version, build_version
            FROM page_impressions
            WHERE user_id = $1 AND page_id = $2
            ORDER BY served_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_impression: {}", e)))?;

        match row {
            Some(r) => r.into_domain().map(Some),
            None => Ok(None),
        }
    }

    async fn get_impression_by_id(
        &self,
        impression_id: Uuid,
    ) -> Result<Option<PageImpression>, DomainError> {
        let row = sqlx::query_as::<_, SqlPageImpression>(
            r#"
            SELECT id, user_id, page_id, served_at, session_id,
                   parent_impression_id, root_page_id, continuation_depth,
                   active_reading_time_ms, scroll_depth, bottom_reached,
                   scroll_back, navigation_action, book_resumed, is_bookmarked,
                   reached_end, app_version, build_version
            FROM page_impressions
            WHERE id = $1
            "#,
        )
        .bind(impression_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_impression_by_id: {}", e)))?;

        match row {
            Some(r) => r.into_domain().map(Some),
            None => Ok(None),
        }
    }

    async fn get_child_impression(
        &self,
        parent_impression_id: Uuid,
        next_page_id: Uuid,
    ) -> Result<Option<PageImpression>, DomainError> {
        let row = sqlx::query_as::<_, SqlPageImpression>(
            r#"
            SELECT id, user_id, page_id, served_at, session_id,
                   parent_impression_id, root_page_id, continuation_depth,
                   active_reading_time_ms, scroll_depth, bottom_reached,
                   scroll_back, navigation_action, book_resumed, is_bookmarked,
                   reached_end, app_version, build_version
            FROM page_impressions
            WHERE parent_impression_id = $1 AND page_id = $2
            ORDER BY served_at ASC
            LIMIT 1
            "#,
        )
        .bind(parent_impression_id)
        .bind(next_page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_child_impression: {}", e)))?;

        match row {
            Some(r) => r.into_domain().map(Some),
            None => Ok(None),
        }
    }

    async fn has_user_read_edition(
        &self,
        user_id: Uuid,
        edition_id: Uuid,
    ) -> Result<bool, DomainError> {
        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM page_impressions pi
                JOIN pages p ON pi.page_id = p.id
                WHERE pi.user_id = $1 AND p.edition_id = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(edition_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL has_user_read_edition: {}", e)))?;

        Ok(row.0)
    }

    async fn get_continuation_chain(
        &self,
        impression_id: Uuid,
    ) -> Result<Vec<PageImpression>, DomainError> {
        let rows = sqlx::query_as::<_, SqlPageImpression>(
            r#"
            WITH RECURSIVE chain AS (
                SELECT id, user_id, page_id, served_at, session_id,
                       parent_impression_id, root_page_id, continuation_depth,
                       active_reading_time_ms, scroll_depth, bottom_reached,
                       scroll_back, navigation_action, book_resumed, is_bookmarked,
                       reached_end, app_version, build_version
                FROM page_impressions
                WHERE id = $1
                UNION ALL
                SELECT p.id, p.user_id, p.page_id, p.served_at, p.session_id,
                       p.parent_impression_id, p.root_page_id, p.continuation_depth,
                       p.active_reading_time_ms, p.scroll_depth, p.bottom_reached,
                       p.scroll_back, p.navigation_action, p.book_resumed, p.is_bookmarked,
                       p.reached_end, p.app_version, p.build_version
                FROM page_impressions p
                INNER JOIN chain c ON c.parent_impression_id = p.id
            )
            SELECT * FROM chain
            ORDER BY continuation_depth ASC
            "#,
        )
        .bind(impression_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_continuation_chain: {}", e)))?;

        rows.into_iter().map(|r| r.into_domain()).collect()
    }

    async fn mark_reached_end(&self, impression_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE page_impressions
            SET reached_end = true
            WHERE id = $1
            "#,
        )
        .bind(impression_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL mark_reached_end: {}", e)))?;

        Ok(())
    }

    async fn has_impression(&self, user_id: Uuid, page_id: Uuid) -> Result<bool, DomainError> {
        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM page_impressions WHERE user_id = $1 AND page_id = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(page_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL has_impression: {}", e)))?;

        Ok(row.0)
    }
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PostgresReactionRepository {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl PostgresReactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlReaction {
    id: Uuid,
    event_id: Uuid,
    user_id: Uuid,
    page_id: Uuid,
    impression_id: Option<Uuid>,
    reaction_type: String,
    reading_time_ms: i32,
    scroll_depth: f64,
    bottom_reached: bool,
    content_overflows: bool,
    navigation_action: Option<String>,
    served_at: DateTime<Utc>,
    reacted_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    app_version: Option<String>,
    build_version: Option<String>,
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlReadingStats {
    pages_served: i64,
    pages_reacted: i64,
    likes: i64,
    dislikes: i64,
    skips: i64,
    saves: i64,
}

#[cfg(feature = "ssr")]
impl ReactionRepository for PostgresReactionRepository {
    async fn record_reaction(&self, reaction: &Reaction) -> Result<bool, DomainError> {
        let nav = reaction.navigation_action.map(|a| a.as_str().to_string());
        let result = sqlx::query(
            r#"
            INSERT INTO reactions (
                id, event_id, user_id, page_id, impression_id, reaction_type,
                reading_time_ms, scroll_depth, bottom_reached, content_overflows,
                navigation_action, served_at, reacted_at, created_at,
                app_version, build_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(reaction.id)
        .bind(reaction.event_id)
        .bind(reaction.user_id)
        .bind(reaction.page_id)
        .bind(reaction.impression_id)
        .bind(reaction.reaction_type.as_str())
        .bind(reaction.reading_time_ms)
        .bind(reaction.scroll_depth)
        .bind(reaction.bottom_reached)
        .bind(reaction.content_overflows)
        .bind(nav)
        .bind(reaction.served_at)
        .bind(reaction.reacted_at)
        .bind(reaction.created_at)
        .bind(reaction.app_version.as_deref())
        .bind(reaction.build_version.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL record_reaction: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_reaction_by_event_id(
        &self,
        event_id: Uuid,
    ) -> Result<Option<Reaction>, DomainError> {
        let row = sqlx::query_as::<_, SqlReaction>(
            r#"
            SELECT id, event_id, user_id, page_id, impression_id, reaction_type,
                   reading_time_ms, scroll_depth, bottom_reached, content_overflows,
                   navigation_action, served_at, reacted_at, created_at,
                   app_version, build_version
            FROM reactions
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_reaction_by_event_id: {}", e)))?;

        match row {
            Some(r) => {
                let reaction_type: ReactionType = r.reaction_type.parse()?;
                let navigation_action =
                    r.navigation_action.as_deref().map(str::parse).transpose()?;

                Ok(Some(Reaction {
                    id: r.id,
                    event_id: r.event_id,
                    user_id: r.user_id,
                    page_id: r.page_id,
                    impression_id: r.impression_id,
                    reaction_type,
                    reading_time_ms: r.reading_time_ms,
                    scroll_depth: r.scroll_depth,
                    bottom_reached: r.bottom_reached,
                    content_overflows: r.content_overflows,
                    navigation_action,
                    served_at: r.served_at,
                    reacted_at: r.reacted_at,
                    created_at: r.created_at,
                    app_version: r.app_version,
                    build_version: r.build_version,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_reaction_by_impression_id(
        &self,
        impression_id: Uuid,
    ) -> Result<Option<Reaction>, DomainError> {
        let row = sqlx::query_as::<_, SqlReaction>(
            r#"
            SELECT id, event_id, user_id, page_id, impression_id, reaction_type,
                   reading_time_ms, scroll_depth, bottom_reached, content_overflows,
                   navigation_action, served_at, reacted_at, created_at,
                   app_version, build_version
            FROM reactions
            WHERE impression_id = $1
            LIMIT 1
            "#,
        )
        .bind(impression_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_reaction_by_impression_id: {}", e)))?;

        match row {
            Some(r) => {
                let reaction_type: ReactionType = r.reaction_type.parse()?;
                let navigation_action =
                    r.navigation_action.as_deref().map(str::parse).transpose()?;

                Ok(Some(Reaction {
                    id: r.id,
                    event_id: r.event_id,
                    user_id: r.user_id,
                    page_id: r.page_id,
                    impression_id: r.impression_id,
                    reaction_type,
                    reading_time_ms: r.reading_time_ms,
                    scroll_depth: r.scroll_depth,
                    bottom_reached: r.bottom_reached,
                    content_overflows: r.content_overflows,
                    navigation_action,
                    served_at: r.served_at,
                    reacted_at: r.reacted_at,
                    created_at: r.created_at,
                    app_version: r.app_version,
                    build_version: r.build_version,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_reaction_for_page(
        &self,
        user_id: Uuid,
        page_id: Uuid,
    ) -> Result<Option<Reaction>, DomainError> {
        let row = sqlx::query_as::<_, SqlReaction>(
            r#"
            SELECT id, event_id, user_id, page_id, impression_id, reaction_type,
                   reading_time_ms, scroll_depth, bottom_reached, content_overflows,
                   navigation_action, served_at, reacted_at, created_at,
                   app_version, build_version
            FROM reactions
            WHERE user_id = $1 AND page_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(page_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_user_reaction_for_page: {}", e)))?;

        match row {
            Some(r) => {
                let reaction_type: ReactionType = r.reaction_type.parse()?;
                let navigation_action =
                    r.navigation_action.as_deref().map(str::parse).transpose()?;

                Ok(Some(Reaction {
                    id: r.id,
                    event_id: r.event_id,
                    user_id: r.user_id,
                    page_id: r.page_id,
                    impression_id: r.impression_id,
                    reaction_type,
                    reading_time_ms: r.reading_time_ms,
                    scroll_depth: r.scroll_depth,
                    bottom_reached: r.bottom_reached,
                    content_overflows: r.content_overflows,
                    navigation_action,
                    served_at: r.served_at,
                    reacted_at: r.reacted_at,
                    created_at: r.created_at,
                    app_version: r.app_version,
                    build_version: r.build_version,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_reading_stats(&self, user_id: Uuid) -> Result<ReadingStats, DomainError> {
        let row = sqlx::query_as::<_, SqlReadingStats>(
            r#"
            SELECT
                COALESCE((SELECT COUNT(*) FROM page_impressions WHERE user_id = $1), 0) AS pages_served,
                COALESCE((SELECT COUNT(*) FROM reactions WHERE user_id = $1), 0) AS pages_reacted,
                COALESCE((SELECT COUNT(*) FROM reactions WHERE user_id = $1 AND reaction_type = 'like'), 0) AS likes,
                COALESCE((SELECT COUNT(*) FROM reactions WHERE user_id = $1 AND reaction_type = 'dislike'), 0) AS dislikes,
                COALESCE((SELECT COUNT(*) FROM reactions WHERE user_id = $1 AND reaction_type = 'skip'), 0) AS skips,
                COALESCE((SELECT COUNT(*) FROM reactions WHERE user_id = $1 AND reaction_type = 'save'), 0) AS saves
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_user_reading_stats: {}", e)))?;

        Ok(ReadingStats {
            user_id,
            pages_served: row.pages_served,
            pages_reacted: row.pages_reacted,
            likes: row.likes,
            dislikes: row.dislikes,
            skips: row.skips,
            saves: row.saves,
        })
    }
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PostgresAffinityRepository {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl PostgresAffinityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlAffinityComparison {
    target_user_id: Uuid,
    common_likes: i64,
    common_dislikes: i64,
    disagreements: i64,
}

#[cfg(feature = "ssr")]
impl AffinityRepository for PostgresAffinityRepository {
    async fn calculate_user_affinities(
        &self,
        user_id: Uuid,
        min_overlap: i32,
        min_score: Option<f64>,
        config: &ReadingValidationConfig,
    ) -> Result<Vec<UserAffinity>, DomainError> {
        // Sélection ciblée des candidats par index inversé sur les pages communes sans O(U^2)
        let rows = sqlx::query_as::<_, SqlAffinityComparison>(
            r#"
            WITH user_reactions AS (
                SELECT page_id, reaction_type
                FROM reactions
                WHERE user_id = $1 AND reaction_type IN ('like', 'dislike')
            ),
            candidates AS (
                SELECT
                    r2.user_id AS target_user_id,
                    COUNT(*) FILTER (WHERE ur.reaction_type = 'like' AND r2.reaction_type = 'like') AS common_likes,
                    COUNT(*) FILTER (WHERE ur.reaction_type = 'dislike' AND r2.reaction_type = 'dislike') AS common_dislikes,
                    COUNT(*) FILTER (WHERE ur.reaction_type <> r2.reaction_type) AS disagreements
                FROM reactions r2
                JOIN user_reactions ur ON ur.page_id = r2.page_id
                WHERE r2.user_id <> $1 AND r2.reaction_type IN ('like', 'dislike')
                GROUP BY r2.user_id
                HAVING COUNT(*) >= $2
            )
            SELECT target_user_id, common_likes, common_dislikes, disagreements
            FROM candidates
            "#,
        )
        .bind(user_id)
        .bind(min_overlap as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL calculate_user_affinities: {}", e)))?;

        let mut affinities: Vec<UserAffinity> = rows
            .into_iter()
            .map(|r| {
                UserAffinity::compute_with_config(
                    r.target_user_id,
                    r.common_likes as i32,
                    r.common_dislikes as i32,
                    r.disagreements as i32,
                    config,
                )
            })
            .collect();

        // Persistance dans user_pair_affinities
        for aff in &affinities {
            let (u_a, u_b) = if user_id < aff.target_user_id {
                (user_id, aff.target_user_id)
            } else {
                (aff.target_user_id, user_id)
            };

            let _ = sqlx::query(
                r#"
                INSERT INTO user_pair_affinities (
                    user_a_id, user_b_id, common_likes, common_dislikes, disagreements,
                    comparable_volume, raw_agreement, confidence, affinity_score, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                ON CONFLICT (user_a_id, user_b_id) DO UPDATE SET
                    common_likes = EXCLUDED.common_likes,
                    common_dislikes = EXCLUDED.common_dislikes,
                    disagreements = EXCLUDED.disagreements,
                    comparable_volume = EXCLUDED.comparable_volume,
                    raw_agreement = EXCLUDED.raw_agreement,
                    confidence = EXCLUDED.confidence,
                    affinity_score = EXCLUDED.affinity_score,
                    updated_at = NOW()
                "#,
            )
            .bind(u_a)
            .bind(u_b)
            .bind(aff.common_likes)
            .bind(aff.common_dislikes)
            .bind(aff.disagreements)
            .bind(aff.comparable_volume)
            .bind(aff.similarity)
            .bind(aff.confidence)
            .bind(aff.affinity_score)
            .execute(&self.pool)
            .await;
        }

        if let Some(min_s) = min_score {
            affinities.retain(|a| a.affinity_score >= min_s);
        }

        affinities.sort_by(|a, b| {
            b.affinity_score
                .partial_cmp(&a.affinity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(affinities)
    }
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PostgresConnectionRepository {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl PostgresConnectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlProposal {
    id: Uuid,
    requester_id: Uuid,
    recipient_id: Uuid,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[cfg(feature = "ssr")]
impl SqlProposal {
    fn into_domain(self) -> Result<ConnectionProposal, DomainError> {
        let status: ConnectionStatus = self.status.parse()?;
        Ok(ConnectionProposal {
            id: self.id,
            requester_id: self.requester_id,
            recipient_id: self.recipient_id,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[cfg(feature = "ssr")]
impl ConnectionRepository for PostgresConnectionRepository {
    async fn request_connection(
        &self,
        requester_id: Uuid,
        recipient_id: Uuid,
    ) -> Result<ConnectionProposal, DomainError> {
        let proposal_id = Uuid::new_v4();
        let row = sqlx::query_as::<_, SqlProposal>(
            r#"
            INSERT INTO connection_proposals (id, requester_id, recipient_id, status, created_at, updated_at)
            VALUES ($1, $2, $3, 'pending', NOW(), NOW())
            ON CONFLICT (requester_id, recipient_id) DO UPDATE
                SET updated_at = NOW()
            RETURNING id, requester_id, recipient_id, status, created_at, updated_at
            "#,
        )
        .bind(proposal_id)
        .bind(requester_id)
        .bind(recipient_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL request_connection: {}", e)))?;

        row.into_domain()
    }

    async fn accept_connection(
        &self,
        requester_id: Uuid,
        recipient_id: Uuid,
    ) -> Result<ConnectionProposal, DomainError> {
        let row = sqlx::query_as::<_, SqlProposal>(
            r#"
            UPDATE connection_proposals
            SET status = 'accepted', updated_at = NOW()
            WHERE (requester_id = $1 AND recipient_id = $2)
               OR (requester_id = $2 AND recipient_id = $1)
            RETURNING id, requester_id, recipient_id, status, created_at, updated_at
            "#,
        )
        .bind(requester_id)
        .bind(recipient_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL accept_connection: {}", e)))?;

        match row {
            Some(r) => r.into_domain(),
            None => Err(DomainError::ConnectionProposalNotFound),
        }
    }

    async fn get_connection_status(
        &self,
        user_a_id: Uuid,
        user_b_id: Uuid,
    ) -> Result<Option<ConnectionProposal>, DomainError> {
        let row = sqlx::query_as::<_, SqlProposal>(
            r#"
            SELECT id, requester_id, recipient_id, status, created_at, updated_at
            FROM connection_proposals
            WHERE (requester_id = $1 AND recipient_id = $2)
               OR (requester_id = $2 AND recipient_id = $1)
            "#,
        )
        .bind(user_a_id)
        .bind(user_b_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::NotFound(format!("SQL get_connection_status: {}", e)))?;

        match row {
            Some(r) => r.into_domain().map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct PostgresAlphaInviteRepository {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl PostgresAlphaInviteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "ssr")]
#[derive(FromRow)]
struct SqlAlphaInviteCode {
    id: Uuid,
    code: String,
    max_uses: i32,
    uses_count: i32,
    is_active: bool,
    note: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "ssr")]
impl AlphaInviteRepository for PostgresAlphaInviteRepository {
    async fn get_invite_by_code(&self, code: &str) -> Result<Option<AlphaInviteCode>, DomainError> {
        let normalized = code.trim().to_uppercase();
        let row = sqlx::query_as::<_, SqlAlphaInviteCode>(
            r#"
            SELECT id, code, max_uses, uses_count, is_active, note, created_at, expires_at
            FROM alpha_invite_codes
            WHERE UPPER(code) = $1
            LIMIT 1
            "#,
        )
        .bind(normalized)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("SQL get_invite_by_code: {}", e)))?;

        Ok(row.map(|r| AlphaInviteCode {
            id: r.id,
            code: r.code,
            max_uses: r.max_uses,
            uses_count: r.uses_count,
            is_active: r.is_active,
            note: r.note,
            created_at: r.created_at,
            expires_at: r.expires_at,
        }))
    }

    async fn claim_invite(
        &self,
        code_id: Uuid,
        user_id: Uuid,
        app_version: Option<&str>,
        build_version: Option<&str>,
        device_summary: Option<&str>,
    ) -> Result<(), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // 1. Ensure user exists in users table
        sqlx::query(
            r#"
            INSERT INTO users (id, created_at)
            VALUES ($1, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("SQL insert user: {}", e)))?;

        // 2. Increment uses_count on invite
        let rows_affected = sqlx::query(
            r#"
            UPDATE alpha_invite_codes
            SET uses_count = uses_count + 1
            WHERE id = $1 AND is_active = true AND uses_count < max_uses
            "#,
        )
        .bind(code_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("SQL update invite: {}", e)))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::InviteCodeExpired(
                "Ce code d'invitation n'est plus actif ou a atteint sa limite".to_string(),
            ));
        }

        // 3. Record claim
        sqlx::query(
            r#"
            INSERT INTO alpha_claimed_invites (id, code_id, user_id, claimed_at, app_version, build_version, device_summary)
            VALUES ($1, $2, $3, NOW(), $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(code_id)
        .bind(user_id)
        .bind(app_version)
        .bind(build_version)
        .bind(device_summary)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("SQL insert claim: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn is_valid_alpha_user(&self, user_id: Uuid) -> Result<bool, DomainError> {
        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM alpha_claimed_invites WHERE user_id = $1
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("SQL is_valid_alpha_user: {}", e)))?;

        Ok(row.0)
    }
}
