use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyLanguageTag);
        }

        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DomainError::InvalidLanguageTag(trimmed.to_string()));
        }

        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLanguagePreference {
    pub user_id: Uuid,
    pub language_tag: LanguageTag,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub original_language_tag: LanguageTag,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edition {
    pub id: Uuid,
    pub work_id: Uuid,
    pub edition_title: String,
    pub translator: Option<String>,
    pub language_tag: LanguageTag,
    pub publication_year: Option<i32>,
    pub publisher: Option<String>,
    pub source_name: String,
    pub source_url: Option<String>,
    pub rights_status: String,
    pub license: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageMetadata {
    pub title: String,
    pub author: String,
    pub original_language_tag: String,
    pub edition_title: String,
    pub translator: Option<String>,
    pub publication_year: Option<i32>,
    pub source_name: String,
}

pub type BookMetadata = PageMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub edition_id: Uuid,
    /// Numéro d'ordre séquentiel obligatoire de la page dans l'édition (1, 2, 3...)
    pub page_number: i32,
    /// Numéro ou libellé de page optionnel provenant de la pagination de la source physique/numérique (ex: "42", "XII")
    pub source_page_number: Option<String>,
    pub content: String,
    /// Hash SHA-256 du texte normalisé servant à l'intégrité et à l'idempotence d'import dans la source
    pub content_hash: String,
    pub language_tag: LanguageTag,
    pub token_count: i32,
    pub random_key: f64,
    pub is_active: bool,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

impl Page {
    pub fn page_sequence_number(&self) -> i32 {
        self.page_number
    }

    pub fn book_id(&self) -> Uuid {
        self.edition_id
    }

    #[cfg(feature = "ssr")]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        edition_id: Uuid,
        page_number: i32,
        source_page_number: Option<String>,
        content: impl Into<String>,
        language_tag: impl Into<String>,
        token_count: i32,
        random_key: f64,
    ) -> Self {
        let content_str = content.into();
        let content_hash = Self::compute_hash(&content_str);
        let lang_str = language_tag.into();
        let lang = LanguageTag::parse(&lang_str).unwrap_or_else(|_| LanguageTag("en".to_string()));
        Self {
            id,
            edition_id,
            page_number,
            source_page_number,
            content: content_str,
            content_hash,
            language_tag: lang,
            token_count,
            random_key,
            is_active: true,
            version: 1,
            created_at: Utc::now(),
        }
    }

    #[cfg(feature = "ssr")]
    pub fn compute_hash(text: &str) -> String {
        use sha2::{Digest, Sha256};
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

pub type Extract = Page;

/// Action de navigation choisie par le lecteur (axe indépendant de la préférence Like/Dislike)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationAction {
    ContinueBook,
    RandomPage,
}

impl NavigationAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            NavigationAction::ContinueBook => "continue_book",
            NavigationAction::RandomPage => "random_page",
        }
    }
}

impl std::str::FromStr for NavigationAction {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "continue_book" | "continue_reading" | "continue_work" | "continue" | "next" => {
                Ok(NavigationAction::ContinueBook)
            }
            "random_page" | "random" => Ok(NavigationAction::RandomPage),
            _ => Err(DomainError::NotFound(format!(
                "Action de navigation invalide: {}",
                s
            ))),
        }
    }
}

impl fmt::Display for NavigationAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageImpression {
    pub id: Uuid,
    pub user_id: Uuid,
    pub page_id: Uuid,
    pub served_at: DateTime<Utc>,
    pub session_id: Option<Uuid>,
    pub parent_impression_id: Option<Uuid>,
    pub root_page_id: Option<Uuid>,
    pub continuation_depth: i32,
    pub active_reading_time_ms: Option<i32>,
    pub scroll_depth: Option<f64>,
    pub bottom_reached: bool,
    pub scroll_back: bool,
    pub navigation_action: Option<NavigationAction>,
    pub book_resumed: bool,
    pub is_bookmarked: bool,
    /// Fait observable neutre indiquant que le lecteur a atteint la dernière page de l'œuvre
    pub reached_end: bool,
    pub app_version: Option<String>,
    pub build_version: Option<String>,
}

impl PageImpression {
    pub fn extract_id(&self) -> Uuid {
        self.page_id
    }
}

pub type ExtractImpression = PageImpression;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadingStats {
    pub user_id: Uuid,
    pub pages_served: i64,
    pub pages_reacted: i64,
    pub likes: i64,
    pub dislikes: i64,
    pub skips: i64,
    pub saves: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingValidationConfig {
    pub min_reading_time_ms: i32,
    pub max_reading_time_ms: i32,
    pub min_server_elapsed_ms: i64,
    /// Seuil de défilement appliqué uniquement si le texte dépasse du viewport
    pub min_scroll_depth: f64,
    pub scroll_required_only_if_overflow: bool,
    /// Seuil indicatif de profil mature (valeur de dev actuelle, non spécification produit définitive)
    pub profile_maturity_threshold: i64,
    pub affinity_half_confidence_k: f64,
    pub min_affinity_connect_threshold: f64,
}

impl Default for ReadingValidationConfig {
    fn default() -> Self {
        Self {
            min_reading_time_ms: 4000,
            max_reading_time_ms: 1_800_000, // 30 minutes max pour nettoyer les sessions oubliées
            min_server_elapsed_ms: 3500,
            min_scroll_depth: 0.75,
            scroll_required_only_if_overflow: true,
            profile_maturity_threshold: 200,
            affinity_half_confidence_k: 20.0,
            min_affinity_connect_threshold: 0.50,
        }
    }
}

pub fn sanitize_reading_time(reading_time_ms: i32, config: &ReadingValidationConfig) -> i32 {
    if reading_time_ms < 0 {
        0
    } else if reading_time_ms > config.max_reading_time_ms {
        config.max_reading_time_ms
    } else {
        reading_time_ms
    }
}

pub fn sanitize_scroll_depth(scroll_depth: f64) -> f64 {
    if scroll_depth.is_nan() || scroll_depth < 0.0 {
        0.0
    } else if scroll_depth > 1.0 {
        1.0
    } else {
        scroll_depth
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionType {
    Like,
    Dislike,
    Skip,
    Save,
    Reveal,
}

impl ReactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReactionType::Like => "like",
            ReactionType::Dislike => "dislike",
            ReactionType::Skip => "skip",
            ReactionType::Save => "save",
            ReactionType::Reveal => "reveal",
        }
    }
}

impl std::str::FromStr for ReactionType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "like" => Ok(ReactionType::Like),
            "dislike" => Ok(ReactionType::Dislike),
            "skip" => Ok(ReactionType::Skip),
            "save" => Ok(ReactionType::Save),
            "reveal" => Ok(ReactionType::Reveal),
            _ => Err(DomainError::InvalidReactionType(s.to_string())),
        }
    }
}

impl fmt::Display for ReactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub const MIN_READING_TIME_MS: i32 = 4000;
pub const MIN_SERVER_ELAPSED_MS: i64 = 3500;
pub const MIN_SCROLL_DEPTH: f64 = 0.75;
pub const AFFINITY_HALF_CONFIDENCE_K: f64 = 20.0;
pub const MIN_AFFINITY_CONNECT_THRESHOLD: f64 = 0.50;

pub fn validate_reading_metrics_full(
    reading_time_ms: i32,
    scroll_depth: f64,
    bottom_reached: bool,
    content_overflows: bool,
    config: &ReadingValidationConfig,
) -> Result<(), DomainError> {
    if reading_time_ms < config.min_reading_time_ms {
        return Err(DomainError::ReadingTooFast {
            reading_time_ms,
            min_required_ms: config.min_reading_time_ms,
        });
    }

    let requires_scroll =
        config.scroll_required_only_if_overflow && content_overflows && !bottom_reached;

    if requires_scroll && scroll_depth < config.min_scroll_depth {
        return Err(DomainError::InsufficientScroll {
            scroll_depth: format!("{:.2}", scroll_depth),
            min_required: format!("{:.2}", config.min_scroll_depth),
        });
    }
    Ok(())
}

pub fn validate_reading_metrics_with_config(
    reading_time_ms: i32,
    scroll_depth: f64,
    config: &ReadingValidationConfig,
) -> Result<(), DomainError> {
    validate_reading_metrics_full(reading_time_ms, scroll_depth, false, true, config)
}

pub fn validate_reading_metrics(
    reading_time_ms: i32,
    scroll_depth: f64,
) -> Result<(), DomainError> {
    validate_reading_metrics_with_config(
        reading_time_ms,
        scroll_depth,
        &ReadingValidationConfig::default(),
    )
}

pub fn validate_server_timing_with_config(
    served_at: DateTime<Utc>,
    now: DateTime<Utc>,
    config: &ReadingValidationConfig,
) -> Result<(), DomainError> {
    let elapsed_ms = (now - served_at).num_milliseconds();
    if elapsed_ms < config.min_server_elapsed_ms {
        return Err(DomainError::ServerTimeElapsedTooShort {
            elapsed_ms,
            min_required_ms: config.min_server_elapsed_ms,
        });
    }
    Ok(())
}

pub fn validate_server_timing(
    served_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(), DomainError> {
    validate_server_timing_with_config(served_at, now, &ReadingValidationConfig::default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub page_id: Uuid,
    pub impression_id: Option<Uuid>,
    pub reaction_type: ReactionType,
    pub reading_time_ms: i32,
    pub scroll_depth: f64,
    pub bottom_reached: bool,
    pub content_overflows: bool,
    pub navigation_action: Option<NavigationAction>,
    pub served_at: DateTime<Utc>,
    pub reacted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub app_version: Option<String>,
    pub build_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaInviteCode {
    pub id: Uuid,
    pub code: String,
    pub max_uses: i32,
    pub uses_count: i32,
    pub is_active: bool,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaClaimedInvite {
    pub id: Uuid,
    pub code_id: Uuid,
    pub user_id: Uuid,
    pub claimed_at: DateTime<Utc>,
    pub app_version: Option<String>,
    pub build_version: Option<String>,
    pub device_summary: Option<String>,
}

impl Reaction {
    pub fn extract_id(&self) -> Uuid {
        self.page_id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAffinity {
    pub target_user_id: Uuid,
    pub common_likes: i32,
    pub common_dislikes: i32,
    pub disagreements: i32,
    pub comparable_volume: i32,
    pub raw_agreement: f64,
    pub similarity: f64,
    pub confidence: f64,
    pub affinity_score: f64,
    pub can_connect: bool,
}

impl UserAffinity {
    pub fn compute_with_config(
        target_user_id: Uuid,
        common_likes: i32,
        common_dislikes: i32,
        disagreements: i32,
        config: &ReadingValidationConfig,
    ) -> Self {
        let comparable_volume = common_likes + common_dislikes + disagreements;
        let n = comparable_volume as f64;
        let agreements = (common_likes + common_dislikes) as f64;

        let (similarity, confidence, affinity_score) = if comparable_volume > 0 {
            let sim = agreements / n;
            let conf = n / (n + config.affinity_half_confidence_k);
            let score = sim * conf;
            (sim, conf, score)
        } else {
            (0.0, 0.0, 0.0)
        };

        let can_connect =
            confidence >= 0.50 && affinity_score >= config.min_affinity_connect_threshold;

        Self {
            target_user_id,
            common_likes,
            common_dislikes,
            disagreements,
            comparable_volume,
            raw_agreement: similarity,
            similarity,
            confidence,
            affinity_score,
            can_connect,
        }
    }

    pub fn compute(
        target_user_id: Uuid,
        common_likes: i32,
        common_dislikes: i32,
        disagreements: i32,
    ) -> Self {
        let default_config = ReadingValidationConfig::default();
        Self::compute_with_config(
            target_user_id,
            common_likes,
            common_dislikes,
            disagreements,
            &default_config,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Pending,
    Accepted,
    Rejected,
}

impl ConnectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionStatus::Pending => "pending",
            ConnectionStatus::Accepted => "accepted",
            ConnectionStatus::Rejected => "rejected",
        }
    }
}

impl std::str::FromStr for ConnectionStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "pending" => Ok(ConnectionStatus::Pending),
            "accepted" => Ok(ConnectionStatus::Accepted),
            "rejected" => Ok(ConnectionStatus::Rejected),
            _ => Err(DomainError::InvalidConnectionStatus(s.to_string())),
        }
    }
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProposal {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub recipient_id: Uuid,
    pub status: ConnectionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConnectionProposal {
    pub fn can_start_discussion(&self) -> bool {
        self.status == ConnectionStatus::Accepted
    }
}
