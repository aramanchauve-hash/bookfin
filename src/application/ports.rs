use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::domain::models::{
    AlphaInviteCode, ConnectionProposal, Page, PageImpression, PageMetadata, Reaction, ReadingStats,
    ReadingValidationConfig, UserAffinity,
};

pub trait PageRepository: Send + Sync {
    fn get_unseen_random(
        &self,
        user_id: Uuid,
        rnd: f64,
    ) -> impl std::future::Future<Output = Result<Option<Page>, DomainError>> + Send;

    fn get_metadata_for_page(
        &self,
        page_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<PageMetadata>, DomainError>> + Send;

    fn get_page_by_id(
        &self,
        page_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Page>, DomainError>> + Send;

    fn get_next_page_in_edition(
        &self,
        edition_id: Uuid,
        current_page_number: i32,
    ) -> impl std::future::Future<Output = Result<Option<Page>, DomainError>> + Send;
}

pub trait ImpressionRepository: Send + Sync {
    fn record_impression(
        &self,
        user_id: Uuid,
        page_id: Uuid,
        session_id: Option<Uuid>,
    ) -> impl std::future::Future<Output = Result<PageImpression, DomainError>> + Send;

    fn record_impression_full(
        &self,
        impression: &PageImpression,
    ) -> impl std::future::Future<Output = Result<PageImpression, DomainError>> + Send;

    fn get_impression(
        &self,
        user_id: Uuid,
        page_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<PageImpression>, DomainError>> + Send;

    fn get_impression_by_id(
        &self,
        impression_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<PageImpression>, DomainError>> + Send;

    fn get_child_impression(
        &self,
        parent_impression_id: Uuid,
        next_page_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<PageImpression>, DomainError>> + Send;

    fn has_user_read_edition(
        &self,
        user_id: Uuid,
        edition_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;

    fn get_continuation_chain(
        &self,
        impression_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<PageImpression>, DomainError>> + Send;

    fn mark_reached_end(
        &self,
        impression_id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn has_impression(
        &self,
        user_id: Uuid,
        page_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;
}

pub trait ReactionRepository: Send + Sync {
    fn record_reaction(
        &self,
        reaction: &Reaction,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;

    fn get_reaction_by_event_id(
        &self,
        event_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Reaction>, DomainError>> + Send;

    fn get_reaction_by_impression_id(
        &self,
        impression_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Reaction>, DomainError>> + Send;

    fn get_user_reaction_for_page(
        &self,
        user_id: Uuid,
        page_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Reaction>, DomainError>> + Send;

    fn get_user_reading_stats(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<ReadingStats, DomainError>> + Send;
}

pub trait AffinityRepository: Send + Sync {
    fn calculate_user_affinities(
        &self,
        user_id: Uuid,
        min_overlap: i32,
        min_score: Option<f64>,
        config: &ReadingValidationConfig,
    ) -> impl std::future::Future<Output = Result<Vec<UserAffinity>, DomainError>> + Send;
}

pub trait ConnectionRepository: Send + Sync {
    fn request_connection(
        &self,
        requester_id: Uuid,
        recipient_id: Uuid,
    ) -> impl std::future::Future<Output = Result<ConnectionProposal, DomainError>> + Send;

    fn accept_connection(
        &self,
        requester_id: Uuid,
        recipient_id: Uuid,
    ) -> impl std::future::Future<Output = Result<ConnectionProposal, DomainError>> + Send;

    fn get_connection_status(
        &self,
        user_a_id: Uuid,
        user_b_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<ConnectionProposal>, DomainError>> + Send;
}

pub trait ExtractRepository: PageRepository {}
impl<T: ?Sized + PageRepository> ExtractRepository for T {}

pub trait AlphaInviteRepository: Send + Sync {
    fn get_invite_by_code(
        &self,
        code: &str,
    ) -> impl std::future::Future<Output = Result<Option<AlphaInviteCode>, DomainError>> + Send;

    fn claim_invite(
        &self,
        code_id: Uuid,
        user_id: Uuid,
        app_version: Option<&str>,
        build_version: Option<&str>,
        device_summary: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn is_valid_alpha_user(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;
}
