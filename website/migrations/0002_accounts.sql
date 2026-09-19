CREATE TABLE lux_accounts (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL COLLATE NOCASE UNIQUE,
  password_hash TEXT NOT NULL,
  password_salt TEXT NOT NULL,
  recovery_hash TEXT NOT NULL,
  social_id TEXT NOT NULL UNIQUE REFERENCES social_users(id),
  preferences TEXT NOT NULL DEFAULT '{}',
  revision INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);
CREATE TABLE lux_sessions (
  token_hash TEXT PRIMARY KEY,
  account_id TEXT NOT NULL REFERENCES lux_accounts(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  client TEXT NOT NULL CHECK (client IN ('web', 'launcher'))
);
CREATE INDEX lux_sessions_account ON lux_sessions(account_id);
CREATE INDEX lux_sessions_expiry ON lux_sessions(expires_at);
