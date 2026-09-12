-- Bookfin Migration 0002: Social Reading Pipeline
-- Non-destructive additions: reading validation metrics, pairwise affinities, and connection proposals.

-- 1. Enrich reactions table with reading attention metrics
ALTER TABLE reactions
    ADD COLUMN IF NOT EXISTS reading_time_ms INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS scroll_depth DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN IF NOT EXISTS served_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS reacted_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- 2. Pairwise affinities table (canonical order user_a_id < user_b_id)
CREATE TABLE IF NOT EXISTS user_pair_affinities (
    user_a_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_b_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    common_likes INT NOT NULL DEFAULT 0,
    common_dislikes INT NOT NULL DEFAULT 0,
    disagreements INT NOT NULL DEFAULT 0,
    comparable_volume INT NOT NULL DEFAULT 0,
    raw_agreement DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    affinity_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_a_id, user_b_id),
    CONSTRAINT chk_user_pair_order CHECK (user_a_id < user_b_id)
);

CREATE INDEX IF NOT EXISTS idx_user_pair_affinities_a ON user_pair_affinities (user_a_id);
CREATE INDEX IF NOT EXISTS idx_user_pair_affinities_b ON user_pair_affinities (user_b_id);
CREATE INDEX IF NOT EXISTS idx_user_pair_affinities_score ON user_pair_affinities (affinity_score DESC);

-- 3. Connection proposals table for mutual acceptance
CREATE TABLE IF NOT EXISTS connection_proposals (
    id UUID PRIMARY KEY,
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_connection_proposal UNIQUE (requester_id, recipient_id),
    CONSTRAINT chk_diff_users CHECK (requester_id <> recipient_id)
);

CREATE INDEX IF NOT EXISTS idx_conn_proposals_recipient ON connection_proposals (recipient_id);
CREATE INDEX IF NOT EXISTS idx_conn_proposals_status ON connection_proposals (status);
