-- Bookfin Migration 0005: Continuation of Reading & Invisible Signals

-- 1. Support real source page numbers alongside Bookfin sequential pagination
ALTER TABLE pages
    ADD COLUMN IF NOT EXISTS source_page_number TEXT;

-- 2. Drop single-impression unique constraint to allow continuation sequences and book resumption
ALTER TABLE page_impressions
    DROP CONSTRAINT IF EXISTS uq_user_page_impression;

-- 3. Add continuation tracking and invisible reading signals to page_impressions
ALTER TABLE page_impressions
    ADD COLUMN IF NOT EXISTS parent_impression_id UUID REFERENCES page_impressions(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS root_page_id UUID REFERENCES pages(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS continuation_depth INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS active_reading_time_ms INT,
    ADD COLUMN IF NOT EXISTS scroll_depth DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS bottom_reached BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS scroll_back BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS navigation_action VARCHAR(32),
    ADD COLUMN IF NOT EXISTS book_resumed BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_bookmarked BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_page_impressions_parent ON page_impressions (parent_impression_id);
CREATE INDEX IF NOT EXISTS idx_page_impressions_user_depth ON page_impressions (user_id, continuation_depth);

-- 4. Add navigation action and presentation flags to reactions
ALTER TABLE reactions
    ADD COLUMN IF NOT EXISTS navigation_action VARCHAR(32),
    ADD COLUMN IF NOT EXISTS bottom_reached BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS content_overflows BOOLEAN NOT NULL DEFAULT true;

