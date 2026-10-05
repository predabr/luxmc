CREATE TABLE IF NOT EXISTS shared_invites (code_hash TEXT PRIMARY KEY, kind TEXT NOT NULL, payload TEXT NOT NULL, expires_at INTEGER NOT NULL);
CREATE INDEX IF NOT EXISTS shared_invites_expiry ON shared_invites(expires_at);
