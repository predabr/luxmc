CREATE TABLE IF NOT EXISTS social_blocks (
  owner TEXT NOT NULL REFERENCES social_users(id) ON DELETE CASCADE,
  target TEXT NOT NULL REFERENCES social_users(id) ON DELETE CASCADE,
  PRIMARY KEY(owner, target),
  CHECK(owner != target)
);
CREATE TABLE IF NOT EXISTS social_rooms (
  code TEXT PRIMARY KEY,
  owner TEXT NOT NULL UNIQUE REFERENCES social_users(id) ON DELETE CASCADE,
  host TEXT NOT NULL,
  port INTEGER NOT NULL CHECK(port BETWEEN 1 AND 65535),
  expires_at INTEGER NOT NULL
);
