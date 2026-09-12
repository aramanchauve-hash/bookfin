-- Migration 0008 : Invitations et versioning closed alpha

CREATE TABLE IF NOT EXISTS alpha_invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    max_uses INT NOT NULL DEFAULT 1,
    uses_count INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_alpha_invite_codes_code ON alpha_invite_codes(code);

CREATE TABLE IF NOT EXISTS alpha_claimed_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code_id UUID NOT NULL REFERENCES alpha_invite_codes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    claimed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    app_version VARCHAR(32),
    build_version VARCHAR(32),
    device_summary TEXT,
    CONSTRAINT uq_alpha_claimed_user UNIQUE(user_id)
);

CREATE INDEX IF NOT EXISTS idx_alpha_claimed_invites_user ON alpha_claimed_invites(user_id);
CREATE INDEX IF NOT EXISTS idx_alpha_claimed_invites_code ON alpha_claimed_invites(code_id);

-- Versioning sur les impressions et réactions pour interprétation des données de test
ALTER TABLE page_impressions ADD COLUMN IF NOT EXISTS app_version VARCHAR(32);
ALTER TABLE page_impressions ADD COLUMN IF NOT EXISTS build_version VARCHAR(32);

ALTER TABLE reactions ADD COLUMN IF NOT EXISTS app_version VARCHAR(32);
ALTER TABLE reactions ADD COLUMN IF NOT EXISTS build_version VARCHAR(32);

