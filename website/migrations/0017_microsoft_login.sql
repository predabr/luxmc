ALTER TABLE lux_accounts ADD COLUMN auth_provider TEXT NOT NULL DEFAULT 'luxmc' CHECK(auth_provider IN ('luxmc', 'microsoft'));
CREATE TABLE lux_microsoft_accounts (
  minecraft_uuid TEXT PRIMARY KEY,
  account_id TEXT NOT NULL UNIQUE REFERENCES lux_accounts(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL
);
CREATE TABLE lux_microsoft_pending (
  state_hash TEXT PRIMARY KEY,
  browser_hash TEXT NOT NULL,
  verifier TEXT NOT NULL,
  social_id TEXT REFERENCES social_users(id),
  expires_at INTEGER NOT NULL
);
CREATE INDEX lux_microsoft_pending_expiry ON lux_microsoft_pending(expires_at);
