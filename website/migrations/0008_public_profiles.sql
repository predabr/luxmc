CREATE TABLE IF NOT EXISTS social_public_profiles (
  user_id TEXT PRIMARY KEY REFERENCES social_users(id) ON DELETE CASCADE,
  description TEXT NOT NULL DEFAULT '',
  banner TEXT NOT NULL DEFAULT '',
  portrait TEXT NOT NULL DEFAULT '',
  packs TEXT NOT NULL DEFAULT '[]',
  updated_at INTEGER NOT NULL
);
