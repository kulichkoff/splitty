CREATE TABLE IF NOT EXISTS parties (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    state VARCHAR(32) NOT NULL DEFAULT 'collecting'
        CHECK (state IN ('collecting', 'locked', 'settled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS members (
    id BIGSERIAL PRIMARY KEY,
    telegram_id BIGINT NOT NULL UNIQUE,
    slug VARCHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS party_members (
    party_id BIGINT NOT NULL REFERENCES parties(id)
        ON DELETE CASCADE,
    member_id BIGINT NOT NULL REFERENCES members(id)
        ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (party_id, member_id)
);

CREATE INDEX idx_party_members_party_id
    ON party_members (party_id);

CREATE INDEX idx_party_members_member_id
    ON party_members (member_id);

CREATE TABLE IF NOT EXISTS expenses (
    id BIGSERIAL PRIMARY KEY,
    amount_cents BIGINT NOT NULL
        CHECK (amount_cents >= 0),
    party_id BIGINT NOT NULL,
    member_id BIGINT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    FOREIGN KEY (party_id, member_id)
        REFERENCES party_members (party_id, member_id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_expenses_member_id
    ON expenses (member_id);

CREATE INDEX idx_expenses_party_id
    ON expenses (party_id);
