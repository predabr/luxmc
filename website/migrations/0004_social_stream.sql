CREATE TABLE IF NOT EXISTS social_stream_tickets (
  token_hash TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES social_users(id) ON DELETE CASCADE,
  expires_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS social_stream_expiry ON social_stream_tickets(expires_at);
