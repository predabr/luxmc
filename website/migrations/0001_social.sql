CREATE TABLE IF NOT EXISTS social_users (
  id TEXT PRIMARY KEY,
  token_hash TEXT NOT NULL UNIQUE,
  username TEXT NOT NULL,
  last_seen INTEGER NOT NULL DEFAULT 0,
  activity TEXT NOT NULL DEFAULT '',
  instance_name TEXT NOT NULL DEFAULT '',
  mc_version TEXT NOT NULL DEFAULT '',
  loader TEXT NOT NULL DEFAULT '',
  server_host TEXT,
  server_port INTEGER
);
CREATE INDEX IF NOT EXISTS social_username ON social_users(username COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS social_relationships (
  sender TEXT NOT NULL REFERENCES social_users(id) ON DELETE CASCADE,
  recipient TEXT NOT NULL REFERENCES social_users(id) ON DELETE CASCADE,
  pair_key TEXT PRIMARY KEY,
  accepted INTEGER NOT NULL DEFAULT 0 CHECK (accepted IN (0, 1)),
  created_at INTEGER NOT NULL,
  CHECK (sender != recipient)
);
CREATE INDEX IF NOT EXISTS social_recipient ON social_relationships(recipient);
CREATE INDEX IF NOT EXISTS social_sender ON social_relationships(sender);
CREATE TABLE IF NOT EXISTS social_limits (
  bucket TEXT PRIMARY KEY,
  count INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
