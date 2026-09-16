use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::{
    BlockV2, ConnectionProposal, PageMetadata, ReactionType, ReadingStats, UserAffinity,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageCardDto {
    pub id: Uuid,
    pub content: String,
    pub language_tag: String,
}

pub type ExtractCardDto = PageCardDto;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageMetadataDto {
    pub title: String,
    pub author: String,
    pub original_language_tag: String,
    pub publication_year: Option<i32>,
}

pub type BookMetadataDto = PageMetadataDto;

impl From<PageMetadata> for PageMetadataDto {
    fn from(m: PageMetadata) -> Self {
        Self {
            title: m.title,
            author: m.author,
            original_language_tag: m.original_language_tag,
            publication_year: m.publication_year,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionTypeDto {
    Like,
    Dislike,
    Skip,
    Save,
    Reveal,
}

impl From<ReactionTypeDto> for ReactionType {
    fn from(dto: ReactionTypeDto) -> Self {
        match dto {
            ReactionTypeDto::Like => ReactionType::Like,
            ReactionTypeDto::Dislike => ReactionType::Dislike,
            ReactionTypeDto::Skip => ReactionType::Skip,
            ReactionTypeDto::Save => ReactionType::Save,
            ReactionTypeDto::Reveal => ReactionType::Reveal,
        }
    }
}

impl From<ReactionType> for ReactionTypeDto {
    fn from(r: ReactionType) -> Self {
        match r {
            ReactionType::Like => ReactionTypeDto::Like,
            ReactionType::Dislike => ReactionTypeDto::Dislike,
            ReactionType::Skip => ReactionTypeDto::Skip,
            ReactionType::Save => ReactionTypeDto::Save,
            ReactionType::Reveal => ReactionTypeDto::Reveal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReactionResponseDto {
    pub success: bool,
    pub metadata: Option<PageMetadataDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedPageDto {
    pub impression_id: Uuid,
    pub page_id: Uuid,
    pub page_sequence_number: i32,
    pub source_page_number: Option<String>,
    pub text: String,
    /// Optional for legacy alpha rows. Curated V1 pages must supply this
    /// renderer-ready V2 structure instead of asking the client to rebuild it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<BlockV2>>,
    pub content_hash: String,
    pub language_tag: String,
    pub token_count: i32,
    pub continuation_depth: i32,
    pub served_at: DateTime<Utc>,
}

impl FeedPageDto {
    pub fn extract_id(&self) -> Uuid {
        self.page_id
    }
    pub fn content(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedNextExtractResponseDto {
    pub extract_id: Uuid,
    pub content: String,
    pub language_tag: String,
    pub token_count: i32,
    pub served_at: DateTime<Utc>,
}

impl From<FeedPageDto> for FeedNextExtractResponseDto {
    fn from(f: FeedPageDto) -> Self {
        Self {
            extract_id: f.page_id,
            content: f.text,
            language_tag: f.language_tag,
            token_count: f.token_count,
            served_at: f.served_at,
        }
    }
}

#[cfg(test)]
mod feed_v2_contract_tests {
    use super::FeedPageDto;
    use crate::domain::models::{BlockTypeV2, BlockV2, SpanV2};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn feed_serializes_structured_v2_blocks_without_flattening() {
        let dto = FeedPageDto {
            impression_id: Uuid::new_v4(), page_id: Uuid::new_v4(), page_sequence_number: 7,
            source_page_number: None, text: "fallback only".into(), content_hash: "a".repeat(64),
            blocks: Some(vec![BlockV2 { block_type: BlockTypeV2::Verse, level: None, spans: vec![SpanV2 { text: "Un vers\nOtro verso".into(), italic: Some(true), bold: None, small_caps: None }] }]),
            language_tag: "fr".into(), token_count: 3, continuation_depth: 0, served_at: Utc::now(),
        };
        let value = serde_json::to_value(dto).unwrap();
        assert_eq!(value["blocks"][0]["type"], "verse");
        assert_eq!(value["blocks"][0]["spans"][0]["text"], "Un vers\nOtro verso");
        assert_eq!(value["blocks"][0]["spans"][0]["italic"], true);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageRevealDto {
    pub page_id: Uuid,
    pub title: String,
    pub author: String,
    pub edition_title: String,
    pub translator: Option<String>,
    pub publication_year: Option<i32>,
    pub source_name: String,
    pub original_language_tag: String,
    pub page_language_tag: String,
    pub has_reacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReactionRequestDto {
    pub event_id: Option<Uuid>,
    pub user_id: Uuid,
    pub page_id: Option<Uuid>,
    pub extract_id: Option<Uuid>,
    pub impression_id: Option<Uuid>,
    pub reaction: String,
    pub reading_time_ms: i32,
    pub scroll_depth: f64,
    pub bottom_reached: Option<bool>,
    pub content_overflows: Option<bool>,
    pub navigation_action: Option<String>,
    pub served_at: Option<DateTime<Utc>>,
    pub reacted_at: Option<DateTime<Utc>>,
    pub app_version: Option<String>,
    pub build_version: Option<String>,
}

impl SubmitReactionRequestDto {
    pub fn resolved_page_id(&self) -> Result<Uuid, String> {
        self.page_id
            .or(self.extract_id)
            .ok_or_else(|| "Champ page_id (ou extract_id) obligatoire".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitReactionResponseDto {
    pub success: bool,
    pub reaction_id: Uuid,
    pub recorded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinueReadingRequestDto {
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub parent_impression_id: Option<Uuid>,
    pub idempotency_key: Option<Uuid>,
    pub app_version: Option<String>,
    pub build_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimAlphaInviteRequestDto {
    pub code: String,
    pub device_summary: Option<String>,
    pub app_version: Option<String>,
    pub build_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimAlphaInviteResponseDto {
    pub user_id: Uuid,
    pub claimed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlphaTesterStatsDto {
    pub user_id: Uuid,
    pub invite_code: String,
    pub note: Option<String>,
    pub claimed_at: DateTime<Utc>,
    pub app_version: Option<String>,
    pub pages_served: i64,
    pub pages_evaluated: i64,
    pub likes: i64,
    pub dislikes: i64,
    pub continue_reading_count: i64,
    pub random_page_count: i64,
    pub max_continuation_depth: i32,
    pub reached_end_count: i64,
    pub resumed_books_count: i64,
    pub total_reading_time_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadingStatsDto {
    pub user_id: Uuid,
    pub pages_served: i64,
    pub pages_reacted: i64,
    pub likes: i64,
    pub dislikes: i64,
    pub skips: i64,
    pub saves: i64,
}

impl From<ReadingStats> for ReadingStatsDto {
    fn from(s: ReadingStats) -> Self {
        Self {
            user_id: s.user_id,
            pages_served: s.pages_served,
            pages_reacted: s.pages_reacted,
            likes: s.likes,
            dislikes: s.dislikes,
            skips: s.skips,
            saves: s.saves,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserAffinityDto {
    pub target_user_id: Uuid,
    pub common_likes: i32,
    pub common_dislikes: i32,
    pub disagreements: i32,
    pub comparable_volume: i32,
    pub similarity: f64,
    pub confidence: f64,
    pub affinity_score: f64,
    pub can_connect: bool,
}

impl From<UserAffinity> for UserAffinityDto {
    fn from(a: UserAffinity) -> Self {
        Self {
            target_user_id: a.target_user_id,
            common_likes: a.common_likes,
            common_dislikes: a.common_dislikes,
            disagreements: a.disagreements,
            comparable_volume: a.comparable_volume,
            similarity: a.similarity,
            confidence: a.confidence,
            affinity_score: a.affinity_score,
            can_connect: a.can_connect,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequestDto {
    pub requester_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionAcceptDto {
    pub recipient_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionProposalDto {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub recipient_id: Uuid,
    pub status: String,
    pub can_start_discussion: bool,
    pub updated_at: DateTime<Utc>,
}

impl From<ConnectionProposal> for ConnectionProposalDto {
    fn from(p: ConnectionProposal) -> Self {
        let can_start_discussion = p.can_start_discussion();
        Self {
            id: p.id,
            requester_id: p.requester_id,
            recipient_id: p.recipient_id,
            status: p.status.to_string(),
            can_start_discussion,
            updated_at: p.updated_at,
        }
    }
}
