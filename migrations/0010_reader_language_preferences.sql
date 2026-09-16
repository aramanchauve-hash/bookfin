-- Corpus/reader V2: the table existed from the prototype, but this index makes
-- the reader-language eligibility check in RANDOM_PAGE cheap and explicit.
CREATE INDEX IF NOT EXISTS idx_user_language_preferences_user_language
    ON user_language_preferences (user_id, language_tag);
