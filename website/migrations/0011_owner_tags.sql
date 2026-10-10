CREATE TABLE IF NOT EXISTS social_user_tags (
    user_id TEXT PRIMARY KEY REFERENCES social_users(id) ON DELETE CASCADE,
    tags TEXT NOT NULL DEFAULT '[]',
    updated_by TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS social_users_presence ON social_users(last_seen, activity);
