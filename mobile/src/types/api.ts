/**
 * Types TypeScript synchronisés avec les DTOs du backend Rust Bookfin.
 */

export interface FeedPageDto {
  impression_id: string;
  id?: string; // alias optionnel pour compatibilité
  page_id: string;
  page_sequence_number: number;
  page_number?: number; // alias optionnel
  source_page_number: string | null;
  text: string;
  content?: string; // alias optionnel
  language_tag: string;
  served_at: string;
  token_count: number;
  continuation_depth?: number;
}

export type ReactionType = 'like' | 'dislike';
export type NavigationAction = 'continue_book' | 'random_page';

export interface SubmitReactionRequestDto {
  user_id: string;
  page_id: string;
  impression_id: string;
  reaction: ReactionType; // 'like' | 'dislike' attendu par Axum
  reading_time_ms: number; // en millisecondes attendu par Axum
  scroll_depth: number; // fraction [0.0..1.0] attendu par Axum (ReadingValidationConfig::min_scroll_depth)
  bottom_reached?: boolean;
  content_overflows?: boolean;
  event_id?: string;
  navigation_action?: NavigationAction | null;
  served_at?: string;
  reacted_at?: string;
  // Aliases optionnels
  reaction_type?: ReactionType;
  dwell_time_ms?: number;
  scroll_depth_percent?: number;
  reading_speed_wpm?: number;
}

export interface SubmitReactionResponseDto {
  success: boolean;
  reaction_id: string;
  recorded: boolean;
  status?: string;
  event_id?: string;
  page_id?: string;
  user_id?: string;
  is_qualified?: boolean;
}

export interface PageRevealDto {
  page_id: string;
  title: string;
  author: string;
  edition_title: string | null;
  translator: string | null;
  publication_year: number | null;
  source_name: string | null;
  original_language_tag?: string;
  page_language_tag?: string;
  has_reacted?: boolean;
  edition_id?: string;
  work_id?: string;
}

export interface ContinueReadingRequestDto {
  user_id: string;
  parent_impression_id?: string | null;
  session_id?: string | null;
  idempotency_key?: string | null;
}

export interface EndOfEditionErrorDto {
  error: 'end_of_edition';
  edition_id: string;
  last_page_number: number;
  message: string;
}
