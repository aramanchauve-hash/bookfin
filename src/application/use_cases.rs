use chrono::Utc;
use uuid::Uuid;

use crate::application::dtos::{
    ClaimAlphaInviteRequestDto, ClaimAlphaInviteResponseDto, ConnectionProposalDto, FeedPageDto,
    PageCardDto, PageRevealDto, ReactionResponseDto, ReactionTypeDto, ReadingStatsDto,
    SubmitReactionRequestDto, SubmitReactionResponseDto, UserAffinityDto, UserLibraryItemDto,
};
use crate::application::ports::{
    AffinityRepository, AlphaInviteRepository, ConnectionRepository, ImpressionRepository,
    PageRepository, ReactionRepository, WorkProgressRepository,
};
use crate::domain::errors::DomainError;
use crate::domain::models::{
    validate_server_timing_with_config, NavigationAction, Page, PageImpression, Reaction,
    ReactionType, ReadingContext, ReadingEvent, ReadingValidationConfig,
};

pub struct FeedNextPageUseCase<P: PageRepository, I: ImpressionRepository> {
    page_repo: P,
    impression_repo: I,
}

impl<P: PageRepository, I: ImpressionRepository> FeedNextPageUseCase<P, I> {
    pub fn new(page_repo: P, impression_repo: I) -> Self {
        Self {
            page_repo,
            impression_repo,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        session_id: Option<Uuid>,
        rnd: f64,
        app_version: Option<String>,
        build_version: Option<String>,
    ) -> Result<Option<FeedPageDto>, DomainError> {
        let page = self.page_repo.get_unseen_random(user_id, rnd).await?;

        match page {
            Some(p) => {
                let now = Utc::now();
                let impression = crate::domain::models::PageImpression {
                    id: Uuid::new_v4(),
                    user_id,
                    page_id: p.id,
                    served_at: now,
                    session_id,
                    parent_impression_id: None,
                    root_page_id: Some(p.id),
                    continuation_depth: 0,
                    active_reading_time_ms: None,
                    scroll_depth: None,
                    bottom_reached: false,
                    scroll_back: false,
                    navigation_action: Some(crate::domain::models::NavigationAction::RandomPage),
                    book_resumed: false,
                    is_bookmarked: false,
                    reached_end: false,
                    app_version,
                    build_version,
                    reading_context: ReadingContext::Discovery,
                };
                let impression = self
                    .impression_repo
                    .record_impression_full(&impression)
                    .await?;

                Ok(Some(FeedPageDto {
                    impression_id: impression.id,
                    page_id: p.id,
                    page_sequence_number: p.page_number,
                    source_page_number: p.source_page_number,
                    text: p.content,
                    blocks: p.content_v2.as_ref().map(|content| content.blocks.clone()),
                    content_hash: p.content_hash,
                    language_tag: p.language_tag.to_string(),
                    token_count: p.token_count,
                    continuation_depth: impression.continuation_depth,
                    served_at: impression.served_at,
                }))
            }
            None => Ok(None),
        }
    }
}

pub type FeedNextExtractUseCase<P, I> = FeedNextPageUseCase<P, I>;

pub struct ContinueReadingUseCase<P: PageRepository, I: ImpressionRepository> {
    page_repo: P,
    impression_repo: I,
}

impl<P: PageRepository, I: ImpressionRepository> ContinueReadingUseCase<P, I> {
    pub fn new(page_repo: P, impression_repo: I) -> Self {
        Self {
            page_repo,
            impression_repo,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        current_page_id: Uuid,
        parent_impression_id: Option<Uuid>,
        session_id: Option<Uuid>,
        app_version: Option<String>,
        build_version: Option<String>,
    ) -> Result<FeedPageDto, DomainError> {
        self.execute_with_context(
            user_id,
            current_page_id,
            parent_impression_id,
            session_id,
            app_version,
            build_version,
        )
        .await
        .map(|(dto, _)| dto)
    }

    /// Comme `execute`, mais retourne aussi le contexte de lecture de la page servie.
    /// Le contexte est hérité de l'impression parente : une continuation depuis le
    /// feed reste de la découverte, une continuation de lecture suivie reste suivie.
    pub async fn execute_with_context(
        &self,
        user_id: Uuid,
        current_page_id: Uuid,
        parent_impression_id: Option<Uuid>,
        session_id: Option<Uuid>,
        app_version: Option<String>,
        build_version: Option<String>,
    ) -> Result<(FeedPageDto, ReadingContext), DomainError> {
        let current_page = self
            .page_repo
            .get_page_by_id(current_page_id)
            .await?
            .ok_or_else(|| DomainError::PageNotFound(current_page_id.to_string()))?;

        // 1. Récupération et validation de l'impression parente
        let parent_impression = match parent_impression_id {
            Some(pid) => {
                let imp = self.impression_repo.get_impression_by_id(pid).await?;
                if let Some(ref i) = imp {
                    if i.user_id != user_id || i.page_id != current_page_id {
                        return Err(DomainError::InvalidContinuation(
                            "L'impression parente ne correspond pas à cet utilisateur ou cette page"
                                .to_string(),
                        ));
                    }
                }
                imp
            }
            None => {
                self.impression_repo
                    .get_impression(user_id, current_page_id)
                    .await?
            }
        };

        // 2. Recherche de la page suivante dans la même édition
        let next_page = self
            .page_repo
            .get_next_page_in_edition(current_page.edition_id, current_page.page_number)
            .await?;

        let p = match next_page {
            Some(p) => p,
            None => {
                if let Some(ref parent) = parent_impression {
                    let _ = self.impression_repo.mark_reached_end(parent.id).await;
                }
                return Err(DomainError::EndOfEdition {
                    edition_id: current_page.edition_id,
                    last_page_number: current_page.page_number,
                });
            }
        };

        // 3. Idempotence réseau : si un retry renvoie la même requête pour le même parent
        if let Some(ref parent) = parent_impression {
            if let Some(existing_child) = self
                .impression_repo
                .get_child_impression(parent.id, p.id)
                .await?
            {
                return Ok((
                    FeedPageDto {
                        impression_id: existing_child.id,
                        page_id: p.id,
                        page_sequence_number: p.page_number,
                        source_page_number: p.source_page_number,
                        text: p.content,
                        blocks: p.content_v2.as_ref().map(|content| content.blocks.clone()),
                        content_hash: p.content_hash,
                        language_tag: p.language_tag.to_string(),
                        token_count: p.token_count,
                        continuation_depth: existing_child.continuation_depth,
                        served_at: existing_child.served_at,
                    },
                    existing_child.reading_context,
                ));
            }
        }

        // 4. Calcul de continuation
        let parent_id = parent_impression.as_ref().map(|i| i.id);
        let root_id = parent_impression
            .as_ref()
            .and_then(|i| i.root_page_id)
            .or(Some(current_page_id));
        let depth = parent_impression
            .as_ref()
            .map(|i| i.continuation_depth + 1)
            .unwrap_or(1);

        // Détection de reprise : l'utilisateur a-t-il déjà lu cette édition auparavant ?
        let book_resumed = if depth == 1 {
            self.impression_repo
                .has_user_read_edition(user_id, current_page.edition_id)
                .await
                .unwrap_or(false)
        } else {
            parent_impression
                .as_ref()
                .map(|i| i.book_resumed)
                .unwrap_or(false)
        };

        let reading_context = parent_impression
            .as_ref()
            .map(|i| i.reading_context)
            .unwrap_or_default();

        let now = Utc::now();
        let impression = crate::domain::models::PageImpression {
            id: Uuid::new_v4(),
            user_id,
            page_id: p.id,
            served_at: now,
            session_id,
            parent_impression_id: parent_id,
            root_page_id: root_id,
            continuation_depth: depth,
            active_reading_time_ms: None,
            scroll_depth: None,
            bottom_reached: false,
            scroll_back: false,
            navigation_action: Some(crate::domain::models::NavigationAction::ContinueBook),
            book_resumed,
            is_bookmarked: false,
            reached_end: false,
            app_version,
            build_version,
            reading_context,
        };

        let recorded = self
            .impression_repo
            .record_impression_full(&impression)
            .await?;

        Ok((
            FeedPageDto {
                impression_id: recorded.id,
                page_id: p.id,
                page_sequence_number: p.page_number,
                source_page_number: p.source_page_number,
                text: p.content,
                blocks: p.content_v2.as_ref().map(|content| content.blocks.clone()),
                content_hash: p.content_hash,
                language_tag: p.language_tag.to_string(),
                token_count: p.token_count,
                continuation_depth: recorded.continuation_depth,
                served_at: recorded.served_at,
            },
            reading_context,
        ))
    }
}

pub struct GetNextExtractUseCase<P: PageRepository, I: ImpressionRepository> {
    feed_use_case: FeedNextPageUseCase<P, I>,
}

impl<P: PageRepository, I: ImpressionRepository> GetNextExtractUseCase<P, I> {
    pub fn new(page_repo: P, impression_repo: I) -> Self {
        Self {
            feed_use_case: FeedNextPageUseCase::new(page_repo, impression_repo),
        }
    }

    pub async fn execute(&self, user_id: Uuid, rnd: f64) -> Result<Option<PageCardDto>, String> {
        match self.feed_use_case.execute(user_id, None, rnd, None, None).await {
            Ok(Some(dto)) => Ok(Some(PageCardDto {
                id: dto.page_id,
                content: dto.text,
                language_tag: dto.language_tag,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct SubmitReactionUseCase<R: ReactionRepository, I: ImpressionRepository> {
    reaction_repo: R,
    impression_repo: I,
}

impl<R: ReactionRepository, I: ImpressionRepository> SubmitReactionUseCase<R, I> {
    pub fn new(reaction_repo: R, impression_repo: I) -> Self {
        Self {
            reaction_repo,
            impression_repo,
        }
    }

    pub async fn execute(
        &self,
        req: SubmitReactionRequestDto,
        config: &ReadingValidationConfig,
    ) -> Result<SubmitReactionResponseDto, DomainError> {
        let page_id = req.resolved_page_id().map_err(DomainError::NotFound)?;

        // 1. Validation du type de réaction : like ou dislike uniquement pour le feed
        let reaction_type = match req.reaction.trim().to_lowercase().as_str() {
            "like" => ReactionType::Like,
            "dislike" => ReactionType::Dislike,
            other => return Err(DomainError::InvalidReactionType(other.to_string())),
        };

        let bottom_reached = req.bottom_reached.unwrap_or(false);
        let content_overflows = req.content_overflows.unwrap_or(true);
        let navigation_action = req
            .navigation_action
            .as_deref()
            .map(str::parse)
            .transpose()?;

        // 2. Validation des métriques client (temps de lecture & scroll si débordement)
        crate::domain::models::validate_reading_metrics_full(
            req.reading_time_ms,
            req.scroll_depth,
            bottom_reached,
            content_overflows,
            config,
        )?;

        // 3. Validation de l'impression pré-existante obligatoire
        let impression = match req.impression_id {
            Some(imp_id) => {
                let imp = self
                    .impression_repo
                    .get_impression_by_id(imp_id)
                    .await?
                    .ok_or_else(|| DomainError::ImpressionNotFound(imp_id.to_string()))?;
                if imp.user_id != req.user_id || imp.page_id != page_id {
                    return Err(DomainError::ImpressionNotFound(
                        "L'impression ne correspond pas à cet utilisateur ou cette page"
                            .to_string(),
                    ));
                }
                imp
            }
            None => self
                .impression_repo
                .get_impression(req.user_id, page_id)
                .await?
                .ok_or_else(|| {
                    DomainError::ImpressionNotFound(
                        "Aucune impression préalable trouvée pour cette page".to_string(),
                    )
                })?,
        };

        // 4. Validation temporelle côté serveur
        let now = Utc::now();
        validate_server_timing_with_config(impression.served_at, now, config)?;

        // 5. Unicité de réaction par impression (AlreadyReacted)
        if let Some(existing) = self
            .reaction_repo
            .get_reaction_by_impression_id(impression.id)
            .await?
        {
            if req.event_id == Some(existing.event_id) {
                // Idempotent retry avec même event_id
                return Ok(SubmitReactionResponseDto {
                    success: true,
                    reaction_id: existing.id,
                    recorded: false,
                });
            } else {
                return Err(DomainError::AlreadyReacted);
            }
        }

        // Sanitization des métriques pour protéger contre sessions oubliées
        let sanitized_reading_time =
            crate::domain::models::sanitize_reading_time(req.reading_time_ms, config);
        let sanitized_scroll_depth = crate::domain::models::sanitize_scroll_depth(req.scroll_depth);

        let event_id = req.event_id.unwrap_or_else(Uuid::new_v4);
        let reaction_id = Uuid::new_v4();

        let reaction = Reaction {
            id: reaction_id,
            event_id,
            user_id: req.user_id,
            page_id,
            impression_id: Some(impression.id),
            reaction_type,
            reading_time_ms: sanitized_reading_time,
            scroll_depth: sanitized_scroll_depth,
            bottom_reached,
            content_overflows,
            navigation_action,
            served_at: impression.served_at,
            reacted_at: req.reacted_at.unwrap_or(now),
            created_at: now,
            app_version: req.app_version,
            build_version: req.build_version,
        };

        let recorded = self.reaction_repo.record_reaction(&reaction).await?;
        let final_reaction_id = if !recorded {
            // Replay sur event_id : récupérer l'id existant
            self.reaction_repo
                .get_reaction_by_event_id(event_id)
                .await?
                .map(|r| r.id)
                .unwrap_or(reaction_id)
        } else {
            reaction_id
        };

        Ok(SubmitReactionResponseDto {
            success: true,
            reaction_id: final_reaction_id,
            recorded,
        })
    }
}

pub type SubmitSocialReactionUseCase<R, I> = SubmitReactionUseCase<R, I>;

pub struct RevealPageMetadataUseCase<P: PageRepository, R: ReactionRepository> {
    page_repo: P,
    reaction_repo: R,
}

impl<P: PageRepository, R: ReactionRepository> RevealPageMetadataUseCase<P, R> {
    pub fn new(page_repo: P, reaction_repo: R) -> Self {
        Self {
            page_repo,
            reaction_repo,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        page_id: Uuid,
    ) -> Result<PageRevealDto, DomainError> {
        let existing_reaction = self
            .reaction_repo
            .get_user_reaction_for_page(user_id, page_id)
            .await?;

        if existing_reaction.is_none() {
            return Err(DomainError::ReactionRequiredForReveal);
        }

        let meta = self
            .page_repo
            .get_metadata_for_page(page_id)
            .await?
            .ok_or_else(|| DomainError::PageNotFound(page_id.to_string()))?;

        let page = self
            .page_repo
            .get_page_by_id(page_id)
            .await?
            .ok_or_else(|| DomainError::PageNotFound(page_id.to_string()))?;

        Ok(PageRevealDto {
            page_id,
            title: meta.title,
            author: meta.author,
            original_language_tag: meta.original_language_tag,
            edition_title: meta.edition_title,
            translator: meta.translator,
            publication_year: meta.publication_year,
            source_name: meta.source_name,
            page_language_tag: page.language_tag.to_string(),
            has_reacted: true,
            work_id: meta.work_id,
            edition_id: meta.edition_id,
            work_progress_page_sequence_number: None,
        })
    }
}

pub struct GetReadingStatsUseCase<R: ReactionRepository> {
    reaction_repo: R,
}

impl<R: ReactionRepository> GetReadingStatsUseCase<R> {
    pub fn new(reaction_repo: R) -> Self {
        Self { reaction_repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<ReadingStatsDto, DomainError> {
        let stats = self.reaction_repo.get_user_reading_stats(user_id).await?;
        Ok(stats.into())
    }
}

pub struct CalculateAffinitiesUseCase<A: AffinityRepository> {
    affinity_repo: A,
}

impl<A: AffinityRepository> CalculateAffinitiesUseCase<A> {
    pub fn new(affinity_repo: A) -> Self {
        Self { affinity_repo }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        min_overlap: i32,
        min_score: Option<f64>,
        config: &ReadingValidationConfig,
    ) -> Result<Vec<UserAffinityDto>, DomainError> {
        let affinities = self
            .affinity_repo
            .calculate_user_affinities(user_id, min_overlap, min_score, config)
            .await?;

        Ok(affinities.into_iter().map(Into::into).collect())
    }
}

pub struct RequestConnectionUseCase<C: ConnectionRepository> {
    connection_repo: C,
}

impl<C: ConnectionRepository> RequestConnectionUseCase<C> {
    pub fn new(connection_repo: C) -> Self {
        Self { connection_repo }
    }

    pub async fn execute(
        &self,
        requester_id: Uuid,
        recipient_id: Uuid,
    ) -> Result<ConnectionProposalDto, DomainError> {
        let proposal = self
            .connection_repo
            .request_connection(requester_id, recipient_id)
            .await?;
        Ok(proposal.into())
    }
}

pub struct AcceptConnectionUseCase<C: ConnectionRepository> {
    connection_repo: C,
}

impl<C: ConnectionRepository> AcceptConnectionUseCase<C> {
    pub fn new(connection_repo: C) -> Self {
        Self { connection_repo }
    }

    pub async fn execute(
        &self,
        requester_id: Uuid,
        recipient_id: Uuid,
    ) -> Result<ConnectionProposalDto, DomainError> {
        let proposal = self
            .connection_repo
            .accept_connection(requester_id, recipient_id)
            .await?;
        Ok(proposal.into())
    }
}

pub struct CanStartDiscussionUseCase<C: ConnectionRepository> {
    connection_repo: C,
}

impl<C: ConnectionRepository> CanStartDiscussionUseCase<C> {
    pub fn new(connection_repo: C) -> Self {
        Self { connection_repo }
    }

    pub async fn execute(
        &self,
        user_a_id: Uuid,
        user_b_id: Uuid,
    ) -> Result<Option<ConnectionProposalDto>, DomainError> {
        let proposal = self
            .connection_repo
            .get_connection_status(user_a_id, user_b_id)
            .await?;
        Ok(proposal.map(Into::into))
    }
}

pub struct RecordReactionUseCase<R: ReactionRepository, P: PageRepository> {
    reaction_repo: R,
    page_repo: P,
}

impl<R: ReactionRepository, P: PageRepository> RecordReactionUseCase<R, P> {
    pub fn new(reaction_repo: R, page_repo: P) -> Self {
        Self {
            reaction_repo,
            page_repo,
        }
    }

    pub async fn execute(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        page_id: Uuid,
        reaction_type_dto: ReactionTypeDto,
    ) -> Result<ReactionResponseDto, String> {
        let reaction_type = ReactionType::from(reaction_type_dto);
        let now = Utc::now();

        let reaction = Reaction {
            id: Uuid::new_v4(),
            event_id,
            user_id,
            page_id,
            impression_id: None,
            reaction_type,
            reading_time_ms: 0,
            scroll_depth: 1.0,
            bottom_reached: true,
            content_overflows: false,
            navigation_action: None,
            served_at: now,
            reacted_at: now,
            created_at: now,
            app_version: None,
            build_version: None,
        };

        self.reaction_repo
            .record_reaction(&reaction)
            .await
            .map_err(|e| e.to_string())?;

        let metadata = if reaction_type == ReactionType::Reveal {
            self.page_repo
                .get_metadata_for_page(page_id)
                .await
                .map_err(|e| e.to_string())?
                .map(Into::into)
        } else {
            None
        };

        Ok(ReactionResponseDto {
            success: true,
            metadata,
        })
    }
}

pub struct ClaimAlphaInviteUseCase<A: AlphaInviteRepository> {
    invite_repo: A,
}

impl<A: AlphaInviteRepository> ClaimAlphaInviteUseCase<A> {
    pub fn new(invite_repo: A) -> Self {
        Self { invite_repo }
    }

    pub async fn execute(
        &self,
        req: ClaimAlphaInviteRequestDto,
    ) -> Result<ClaimAlphaInviteResponseDto, DomainError> {
        let code = req.code.trim();
        if code.is_empty() {
            return Err(DomainError::InvalidInviteCode(
                "Le code d'invitation est vide".to_string(),
            ));
        }

        let invite = self
            .invite_repo
            .get_invite_by_code(code)
            .await?
            .ok_or_else(|| DomainError::InvalidInviteCode(code.to_string()))?;

        if !invite.is_active {
            return Err(DomainError::InvalidInviteCode(
                "Ce code d'invitation a été révoqué".to_string(),
            ));
        }

        if let Some(exp) = invite.expires_at {
            if exp < Utc::now() {
                return Err(DomainError::InviteCodeExpired(format!(
                    "Ce code d'invitation a expiré le {}",
                    exp
                )));
            }
        }

        if invite.uses_count >= invite.max_uses {
            return Err(DomainError::InviteCodeExpired(
                "Ce code d'invitation a atteint son nombre maximal d'utilisations".to_string(),
            ));
        }

        let user_id = Uuid::new_v4();
        self.invite_repo
            .claim_invite(
                invite.id,
                user_id,
                req.app_version.as_deref(),
                req.build_version.as_deref(),
                req.device_summary.as_deref(),
            )
            .await?;

        Ok(ClaimAlphaInviteResponseDto {
            user_id,
            claimed: true,
            message: "Invitation validée avec succès".to_string(),
        })
    }
}

pub struct VerifyAlphaUserUseCase<A: AlphaInviteRepository> {
    invite_repo: A,
}

impl<A: AlphaInviteRepository> VerifyAlphaUserUseCase<A> {
    pub fn new(invite_repo: A) -> Self {
        Self { invite_repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<bool, DomainError> {
        self.invite_repo.is_valid_alpha_user(user_id).await
    }
}

/// Enregistre une impression de lecture suivie (jamais une impression aléatoire).
fn sequential_impression(user_id: Uuid, page: &Page, action: NavigationAction) -> PageImpression {
    PageImpression {
        id: Uuid::new_v4(),
        user_id,
        page_id: page.id,
        served_at: Utc::now(),
        session_id: None,
        parent_impression_id: None,
        root_page_id: Some(page.id),
        continuation_depth: 0,
        active_reading_time_ms: None,
        scroll_depth: None,
        bottom_reached: false,
        scroll_back: false,
        navigation_action: Some(action),
        book_resumed: action == NavigationAction::ResumeWork,
        is_bookmarked: false,
        reached_end: false,
        app_version: None,
        build_version: None,
        reading_context: ReadingContext::Sequential,
    }
}

fn feed_page_dto(page: Page, impression: &PageImpression) -> FeedPageDto {
    FeedPageDto {
        impression_id: impression.id,
        page_id: page.id,
        page_sequence_number: page.page_number,
        source_page_number: page.source_page_number,
        text: page.content,
        blocks: page.content_v2.map(|c| c.blocks),
        content_hash: page.content_hash,
        language_tag: page.language_tag.to_string(),
        token_count: page.token_count,
        continuation_depth: impression.continuation_depth,
        served_at: impression.served_at,
    }
}

/// START_WORK / RESTART.
///
/// Sémantique :
/// - œuvre jamais commencée : crée la progression à la page 1 (`start_work`) ;
/// - œuvre déjà commencée, `restart = false` : `WorkAlreadyStarted`, rien n'est modifié ;
/// - `restart = true` (confirmé par le lecteur) : remet la progression à la page 1 (`restart_work`).
pub struct StartWorkUseCase<P: PageRepository, I: ImpressionRepository, W: WorkProgressRepository> {
    page_repo: P,
    impression_repo: I,
    work_progress_repo: W,
}

impl<P: PageRepository, I: ImpressionRepository, W: WorkProgressRepository> StartWorkUseCase<P, I, W> {
    pub fn new(page_repo: P, impression_repo: I, work_progress_repo: W) -> Self {
        Self {
            page_repo,
            impression_repo,
            work_progress_repo,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        work_id: Uuid,
        edition_id: Uuid,
        restart: bool,
    ) -> Result<FeedPageDto, DomainError> {
        if !self
            .work_progress_repo
            .edition_belongs_to_work(work_id, edition_id)
            .await?
        {
            return Err(DomainError::NotFound(format!(
                "Édition {} introuvable pour l'œuvre {}",
                edition_id, work_id
            )));
        }

        let first_page = self
            .page_repo
            .get_first_page_of_edition(edition_id)
            .await?
            .ok_or_else(|| DomainError::PageNotFound(format!("Page 1 introuvable pour l'édition {}", edition_id)))?;

        let event_type = if restart {
            self.work_progress_repo
                .reset_progress(user_id, work_id, edition_id, first_page.id)
                .await?;
            "restart_work"
        } else {
            match self
                .work_progress_repo
                .create_progress_if_absent(user_id, work_id, edition_id, first_page.id)
                .await?
            {
                Some(_) => "start_work",
                None => {
                    let current = self
                        .work_progress_repo
                        .get_progress(user_id, work_id)
                        .await?
                        .map(|p| p.current_page_sequence_number)
                        .unwrap_or(1);
                    return Err(DomainError::WorkAlreadyStarted {
                        current_page_sequence_number: current,
                    });
                }
            }
        };

        let event = ReadingEvent {
            id: Uuid::new_v4(),
            user_id,
            work_id,
            edition_id: Some(edition_id),
            page_id: Some(first_page.id),
            event_type: event_type.to_string(),
            page_sequence_number: Some(first_page.page_number),
            created_at: Utc::now(),
        };
        let _ = self.work_progress_repo.record_reading_event(&event).await;

        let impression = self
            .impression_repo
            .record_impression_full(&sequential_impression(user_id, &first_page, NavigationAction::StartWork))
            .await?;

        Ok(feed_page_dto(first_page, &impression))
    }
}

/// RESUME_WORK : ressert la page de la progression enregistrée, sans la modifier.
pub struct ResumeWorkUseCase<P: PageRepository, I: ImpressionRepository, W: WorkProgressRepository> {
    page_repo: P,
    impression_repo: I,
    work_progress_repo: W,
}

impl<P: PageRepository, I: ImpressionRepository, W: WorkProgressRepository> ResumeWorkUseCase<P, I, W> {
    pub fn new(page_repo: P, impression_repo: I, work_progress_repo: W) -> Self {
        Self {
            page_repo,
            impression_repo,
            work_progress_repo,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        work_id: Uuid,
    ) -> Result<FeedPageDto, DomainError> {
        let progress = self
            .work_progress_repo
            .get_progress(user_id, work_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Aucune progression trouvée pour l'œuvre {}", work_id)))?;

        let page = self
            .page_repo
            .get_page_by_edition_and_number(progress.edition_id, progress.current_page_sequence_number)
            .await?
            .ok_or_else(|| DomainError::PageNotFound(format!("Page {} introuvable pour l'édition {}", progress.current_page_sequence_number, progress.edition_id)))?;

        let event = ReadingEvent {
            id: Uuid::new_v4(),
            user_id,
            work_id,
            edition_id: Some(progress.edition_id),
            page_id: Some(page.id),
            event_type: "resume_work".to_string(),
            page_sequence_number: Some(page.page_number),
            created_at: Utc::now(),
        };
        let _ = self.work_progress_repo.record_reading_event(&event).await;

        let impression = self
            .impression_repo
            .record_impression_full(&sequential_impression(user_id, &page, NavigationAction::ResumeWork))
            .await?;

        Ok(feed_page_dto(page, &impression))
    }
}

/// Après une continuation, avance la progression de l'œuvre UNIQUEMENT si la page
/// a été servie en lecture suivie. Une continuation du feed aléatoire ne touche
/// jamais `user_work_progress`, même si le lecteur a commencé cette œuvre.
pub async fn sync_sequential_progress<P: PageRepository, W: WorkProgressRepository>(
    page_repo: &P,
    work_progress_repo: &W,
    user_id: Uuid,
    served: &FeedPageDto,
    reading_context: ReadingContext,
) -> Result<bool, DomainError> {
    if reading_context != ReadingContext::Sequential {
        return Ok(false);
    }
    let Some(page) = page_repo.get_page_by_id(served.page_id).await? else {
        return Ok(false);
    };
    work_progress_repo
        .update_progress(user_id, page.edition_id, page.id, page.page_number)
        .await
}

pub struct GetLibraryUseCase<W: WorkProgressRepository> {
    work_progress_repo: W,
}

impl<W: WorkProgressRepository> GetLibraryUseCase<W> {
    pub fn new(work_progress_repo: W) -> Self {
        Self { work_progress_repo }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserLibraryItemDto>, DomainError> {
        self.work_progress_repo.get_user_library(user_id).await
    }
}
