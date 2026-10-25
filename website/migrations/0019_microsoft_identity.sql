CREATE TABLE lux_microsoft_subjects (
  subject_hash TEXT PRIMARY KEY,
  account_id TEXT NOT NULL UNIQUE REFERENCES lux_accounts(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL
);
CREATE TABLE lux_microsoft_link_pending (
  token_hash TEXT PRIMARY KEY,
  subject_hash TEXT NOT NULL,
  display_name TEXT NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX lux_microsoft_link_pending_expiry ON lux_microsoft_link_pending(expires_at);
