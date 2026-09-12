-- Bookfin Migration 0007: Reached End / Completed Work Signal
-- Records the observable fact when a reader reaches the final page of an edition/work.

ALTER TABLE page_impressions
    ADD COLUMN IF NOT EXISTS reached_end BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_page_impressions_reached_end 
    ON page_impressions (user_id, reached_end) WHERE reached_end = true;

