CREATE TABLE lux_memberships (
    social_id TEXT PRIMARY KEY REFERENCES social_users(id),
    plan TEXT NOT NULL CHECK(plan IN ('plus', 'pro')),
    expires_at INTEGER NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1,
    change_id TEXT
);
CREATE TABLE lux_purchase_orders (
    id TEXT PRIMARY KEY,
    social_id TEXT NOT NULL REFERENCES social_users(id),
    plan TEXT NOT NULL CHECK(plan IN ('plus', 'pro')),
    amount INTEGER NOT NULL,
    bank TEXT NOT NULL,
    pix_key TEXT NOT NULL,
    days INTEGER NOT NULL DEFAULT 30,
    status TEXT NOT NULL DEFAULT 'awaiting_receipt' CHECK(status IN ('awaiting_receipt','pending','approved','rejected','cancelled')),
    version INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    reviewed_by TEXT,
    decision_id TEXT UNIQUE,
    transaction_id TEXT UNIQUE,
    note TEXT NOT NULL DEFAULT ''
);
CREATE UNIQUE INDEX lux_purchase_active ON lux_purchase_orders(social_id) WHERE status IN ('awaiting_receipt','pending');
CREATE INDEX lux_purchase_queue ON lux_purchase_orders(status, created_at);
CREATE INDEX lux_purchase_customer ON lux_purchase_orders(social_id, created_at);
CREATE TABLE lux_purchase_receipts (
    order_id TEXT PRIMARY KEY REFERENCES lux_purchase_orders(id),
    bytes BLOB NOT NULL,
    mime TEXT NOT NULL,
    size INTEGER NOT NULL CHECK(size BETWEEN 12 AND 1048576),
    sha256 TEXT NOT NULL,
    uploaded_at INTEGER NOT NULL
);
CREATE TABLE lux_purchase_audit (
    id TEXT PRIMARY KEY,
    order_id TEXT REFERENCES lux_purchase_orders(id),
    social_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    detail TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE lux_membership_preferences (
    social_id TEXT PRIMARY KEY REFERENCES social_users(id),
    preferences TEXT NOT NULL DEFAULT '{}',
    presets TEXT NOT NULL DEFAULT '[]',
    revision INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE lux_commerce_tickets (
    token_hash TEXT PRIMARY KEY,
    social_id TEXT NOT NULL REFERENCES social_users(id),
    expires_at INTEGER NOT NULL
);
CREATE TABLE lux_commerce_sessions (
    token_hash TEXT PRIMARY KEY,
    social_id TEXT NOT NULL REFERENCES social_users(id),
    expires_at INTEGER NOT NULL
);
CREATE INDEX lux_commerce_session_expiry ON lux_commerce_sessions(expires_at);
CREATE TABLE lux_commerce_config (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    enabled INTEGER NOT NULL DEFAULT 0,
    bank TEXT NOT NULL,
    pix_key TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT 0
);
INSERT INTO lux_commerce_config(id, bank, pix_key) VALUES(1, 'Nubank', '06a84b9e-71b9-4c36-9ebd-07105ce54808');
