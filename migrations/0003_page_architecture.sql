-- Bookfin Migration 0003: Page Architecture
-- Introduces Works, Editions, Pages, PageImpressions, and associates Reactions with Pages.
-- Fully non-destructive and backfills existing books/extracts data.

-- 1. Works Table (Abstract literary work)
CREATE TABLE IF NOT EXISTS works (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    original_language_tag VARCHAR(32) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Editions Table (Specific edition, translation or digital source of a work)
CREATE TABLE IF NOT EXISTS editions (
    id UUID PRIMARY KEY,
    work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    edition_title TEXT NOT NULL,
    translator TEXT,
    language_tag VARCHAR(32) NOT NULL,
    publication_year INT,
    publisher TEXT,
    source_name TEXT NOT NULL DEFAULT 'Initial Corpus',
    source_url TEXT,
    rights_status TEXT NOT NULL DEFAULT 'public_domain',
    license TEXT NOT NULL DEFAULT 'public_domain',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_editions_work ON editions (work_id);
CREATE INDEX IF NOT EXISTS idx_editions_lang ON editions (language_tag);

-- 3. Pages Table (Immutable pages of an edition)
CREATE TABLE IF NOT EXISTS pages (
    id UUID PRIMARY KEY,
    edition_id UUID NOT NULL REFERENCES editions(id) ON DELETE CASCADE,
    page_number INT NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    language_tag VARCHAR(32) NOT NULL,
    token_count INT NOT NULL,
    random_key DOUBLE PRECISION NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_edition_page_number UNIQUE (edition_id, page_number)
);

CREATE INDEX IF NOT EXISTS idx_pages_random_key ON pages (random_key);
CREATE INDEX IF NOT EXISTS idx_pages_lang_active_random ON pages (language_tag, is_active, random_key);
CREATE INDEX IF NOT EXISTS idx_pages_content_hash ON pages (content_hash);
CREATE INDEX IF NOT EXISTS idx_pages_edition ON pages (edition_id);

-- 4. Page Impressions Table
CREATE TABLE IF NOT EXISTS page_impressions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    page_id UUID NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    served_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id UUID,
    CONSTRAINT uq_user_page_impression UNIQUE (user_id, page_id)
);

CREATE INDEX IF NOT EXISTS idx_page_impressions_user_page ON page_impressions (user_id, page_id);
CREATE INDEX IF NOT EXISTS idx_page_impressions_served_at ON page_impressions (served_at);

-- 5. Enrich Reactions Table to point to Page and Impression
ALTER TABLE reactions
    ADD COLUMN IF NOT EXISTS page_id UUID REFERENCES pages(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS impression_id UUID REFERENCES page_impressions(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_reactions_page_id ON reactions (page_id);
CREATE INDEX IF NOT EXISTS idx_reactions_user_page ON reactions (user_id, page_id);
-- Fast candidate selection index for affinity calculations:
CREATE INDEX IF NOT EXISTS idx_reactions_page_user_type ON reactions (page_id, user_id, reaction_type);

-- 6. Backfill existing books into works & editions
INSERT INTO works (id, title, author, original_language_tag, created_at)
SELECT id, title, author, original_language_tag, created_at FROM books
ON CONFLICT (id) DO NOTHING;

INSERT INTO editions (id, work_id, edition_title, translator, language_tag, publication_year, source_name, rights_status, license, is_active, created_at)
SELECT id, id, title, NULL, original_language_tag, publication_year, 'Initial Corpus', 'public_domain', 'public_domain', true, created_at FROM books
ON CONFLICT (id) DO NOTHING;

-- 7. Backfill existing extracts into pages
INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key, is_active, version, created_at)
SELECT 
    id, 
    book_id, 
    ROW_NUMBER() OVER (PARTITION BY book_id ORDER BY id),
    content, 
    MD5(content), 
    language_tag, 
    token_count, 
    random_key, 
    true, 
    1, 
    created_at
FROM extracts
ON CONFLICT (id) DO NOTHING;

-- 8. Backfill existing impressions into page_impressions
INSERT INTO page_impressions (id, user_id, page_id, served_at)
SELECT id, user_id, extract_id, shown_at
FROM extract_impressions
ON CONFLICT (user_id, page_id) DO NOTHING;

-- 9. Backfill page_id in reactions
UPDATE reactions SET page_id = extract_id WHERE page_id IS NULL;