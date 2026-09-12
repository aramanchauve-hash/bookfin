-- Bookfin Initial Schema (0001_initial_schema.sql)
-- Strict separation: No development seeds in this migration!

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_language_preferences (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    language_tag VARCHAR(32) NOT NULL,
    priority INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, language_tag),
    CONSTRAINT uq_user_lang_priority UNIQUE (user_id, priority)
);

CREATE TABLE IF NOT EXISTS books (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    original_language_tag VARCHAR(32) NOT NULL,
    publication_year INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS extracts (
    id UUID PRIMARY KEY,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    language_tag VARCHAR(32) NOT NULL,
    token_count INT NOT NULL,
    random_key DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_extracts_random_key ON extracts (random_key);
CREATE INDEX IF NOT EXISTS idx_extracts_lang_random ON extracts (language_tag, random_key);

CREATE TABLE IF NOT EXISTS extract_impressions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    extract_id UUID NOT NULL REFERENCES extracts(id) ON DELETE CASCADE,
    shown_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_extract_impression UNIQUE (user_id, extract_id)
);

CREATE INDEX IF NOT EXISTS idx_impressions_user_extract ON extract_impressions (user_id, extract_id);

CREATE TABLE IF NOT EXISTS reactions (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    extract_id UUID NOT NULL REFERENCES extracts(id) ON DELETE CASCADE,
    reaction_type VARCHAR(16) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reactions_user_extract ON reactions (user_id, extract_id);
CREATE INDEX IF NOT EXISTS idx_reactions_event_id ON reactions (event_id);

CREATE TABLE IF NOT EXISTS taste_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    profile_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_affinities (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dimension VARCHAR(64) NOT NULL,
    affinity_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, dimension)
);

