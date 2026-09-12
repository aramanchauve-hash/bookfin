-- Metadata required to audit a long-form public-domain corpus without
-- conflating an abstract work, a specific source edition, and Bookfin pages.
ALTER TABLE works
    ADD COLUMN IF NOT EXISTS work_type TEXT NOT NULL DEFAULT 'narrative';

ALTER TABLE editions
    ADD COLUMN IF NOT EXISTS provenance TEXT,
    ADD COLUMN IF NOT EXISTS rights_basis TEXT,
    ADD COLUMN IF NOT EXISTS source_text_sha256 VARCHAR(64);

CREATE INDEX IF NOT EXISTS idx_editions_source_text_sha256
    ON editions (source_text_sha256)
    WHERE source_text_sha256 IS NOT NULL;
