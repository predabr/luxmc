ALTER TABLE social_users ADD COLUMN removed_at INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS social_active_presence ON social_users(removed_at, last_seen);
