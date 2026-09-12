-- Bookfin Migration 0006: Hardening, Integrity & Immutability
-- Implements strict constraints, missing indexes on hot paths, and triggers preventing historical page mutation.

-- 1. Missing Indexes on Hot Query Paths
CREATE INDEX IF NOT EXISTS idx_page_impressions_user_page ON page_impressions (user_id, page_id);
CREATE INDEX IF NOT EXISTS idx_page_impressions_page_id ON page_impressions (page_id);
CREATE INDEX IF NOT EXISTS idx_page_impressions_served_at ON page_impressions (served_at);
CREATE INDEX IF NOT EXISTS idx_reactions_page_id ON reactions (page_id);
CREATE INDEX IF NOT EXISTS idx_reactions_user_id ON reactions (user_id);

-- 2. Idempotence & Unique Reaction per Impression
CREATE UNIQUE INDEX IF NOT EXISTS uq_reactions_impression_id ON reactions (impression_id) WHERE impression_id IS NOT NULL;

-- 3. Versioned Edition Uniqueness
ALTER TABLE pages DROP CONSTRAINT IF EXISTS uq_edition_page_number;
CREATE UNIQUE INDEX IF NOT EXISTS uq_edition_page_version ON pages (edition_id, page_number, version);
CREATE UNIQUE INDEX IF NOT EXISTS uq_edition_page_active ON pages (edition_id, page_number) WHERE is_active = true;

-- 4. Strict Domain Check Constraints
UPDATE pages SET content_hash = rpad(content_hash, 64, '0') WHERE length(content_hash) <> 64;
UPDATE pages SET page_number = 1 WHERE page_number <= 0;
UPDATE pages SET random_key = 0.5 WHERE random_key < 0.0 OR random_key >= 1.0;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_page_number') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_page_number CHECK (page_number > 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_token_count') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_token_count CHECK (token_count >= 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_random_key') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_random_key CHECK (random_key >= 0.0 AND random_key < 1.0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_content_hash') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_content_hash CHECK (length(content_hash) = 64);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_version') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_version CHECK (version >= 1);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_works_lang') THEN
        ALTER TABLE works ADD CONSTRAINT chk_works_lang CHECK (length(original_language_tag) >= 2);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_editions_lang') THEN
        ALTER TABLE editions ADD CONSTRAINT chk_editions_lang CHECK (length(language_tag) >= 2);
    END IF;
END $$;

-- 5. Immutability Trigger: Prohibit Mutation of Historically Served Pages
CREATE OR REPLACE FUNCTION prevent_page_content_mutation()
RETURNS TRIGGER AS $$
BEGIN
    IF (NEW.content <> OLD.content OR NEW.edition_id <> OLD.edition_id OR NEW.page_number <> OLD.page_number) THEN
        IF EXISTS (SELECT 1 FROM page_impressions WHERE page_id = OLD.id)
           OR EXISTS (SELECT 1 FROM reactions WHERE page_id = OLD.id) THEN
            RAISE EXCEPTION 'ImmutablePageViolation: Impossible de modifier le texte ou les coordonnées d''une page déjà servie (page_id=%). Créez une nouvelle page avec version incrémentée.', OLD.id;
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_page_content_mutation ON pages;
CREATE TRIGGER trg_prevent_page_content_mutation
BEFORE UPDATE ON pages
FOR EACH ROW
EXECUTE FUNCTION prevent_page_content_mutation();
