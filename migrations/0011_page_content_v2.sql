-- Curated V1 semantic page payloads.
-- Additive only: pre-existing alpha pages may retain NULL content_v2.

ALTER TABLE pages
    ADD COLUMN IF NOT EXISTS content_v2 JSONB,
    ADD COLUMN IF NOT EXISTS canonical_content_hash VARCHAR(64);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_content_v2_shape') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_content_v2_shape CHECK (
            content_v2 IS NULL
            OR (jsonb_typeof(content_v2) = 'object'
                AND content_v2 ? 'blocks'
                AND jsonb_typeof(content_v2->'blocks') = 'array')
        );
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_pages_canonical_content_hash') THEN
        ALTER TABLE pages ADD CONSTRAINT chk_pages_canonical_content_hash CHECK (
            canonical_content_hash IS NULL OR length(canonical_content_hash) = 64
        );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_pages_content_v2_present
    ON pages (id)
    WHERE content_v2 IS NOT NULL;
