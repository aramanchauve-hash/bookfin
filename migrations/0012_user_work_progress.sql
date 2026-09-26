-- Bookfin Migration 0012: User Work Progress, Reading Events & Reading Context
-- Supports followed reading (lecture suivie) and the personal library, while
-- keeping discovery (random feed) and sequential reading as distinct contexts.
-- Strictly additive and non-destructive: existing rows keep their meaning.

-- 1. Reading context of each impression.
-- Every impression recorded before this migration came from the discovery feed
-- or its continuations, hence the 'discovery' default. Sequential reading
-- (START_WORK, RESUME_WORK and the pages that follow) is tagged 'sequential' so
-- it never counts as a RANDOM_PAGE_IMPRESSION.
ALTER TABLE page_impressions
    ADD COLUMN IF NOT EXISTS reading_context VARCHAR(16) NOT NULL DEFAULT 'discovery';

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_page_impressions_reading_context') THEN
        ALTER TABLE page_impressions ADD CONSTRAINT chk_page_impressions_reading_context
            CHECK (reading_context IN ('discovery', 'sequential'));
    END IF;
END $$;

-- 2. Current position of a reader in a work they deliberately started.
-- One row per (reader, work). Only sequential reading may move it.
CREATE TABLE IF NOT EXISTS user_work_progress (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    edition_id UUID NOT NULL REFERENCES editions(id) ON DELETE CASCADE,
    current_page_id UUID REFERENCES pages(id) ON DELETE SET NULL,
    current_page_sequence_number INT NOT NULL DEFAULT 1
        CONSTRAINT chk_user_work_progress_page_number CHECK (current_page_sequence_number > 0),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, work_id)
);

CREATE INDEX IF NOT EXISTS idx_user_work_progress_user_last_read
    ON user_work_progress (user_id, last_read_at DESC);

-- 3. Raw, append-only work-level events (future analyses only; no ranking).
CREATE TABLE IF NOT EXISTS reading_events (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    edition_id UUID NOT NULL REFERENCES editions(id) ON DELETE CASCADE,
    page_id UUID REFERENCES pages(id) ON DELETE SET NULL,
    event_type VARCHAR(32) NOT NULL
        CONSTRAINT chk_reading_events_type CHECK (event_type IN ('start_work', 'resume_work', 'restart_work')),
    page_sequence_number INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reading_events_user_work
    ON reading_events (user_id, work_id, created_at);
