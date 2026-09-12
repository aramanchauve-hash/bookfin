-- Bookfin Migration 0004: Make extract_id optional for Page-first reactions
-- Allows reactions to be recorded directly on pages without requiring a legacy extract row.

ALTER TABLE reactions DROP CONSTRAINT IF EXISTS reactions_extract_id_fkey;
ALTER TABLE reactions ALTER COLUMN extract_id DROP NOT NULL;

