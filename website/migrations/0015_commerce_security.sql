ALTER TABLE lux_commerce_sessions ADD COLUMN verified_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE lux_commerce_sessions ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE lux_commerce_sessions ADD COLUMN last_activity_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE lux_commerce_config ADD COLUMN owner_totp_secret TEXT;
ALTER TABLE lux_commerce_config ADD COLUMN owner_totp_pending TEXT;
ALTER TABLE lux_commerce_config ADD COLUMN owner_totp_pending_until INTEGER NOT NULL DEFAULT 0;
ALTER TABLE lux_commerce_config ADD COLUMN owner_totp_last_step INTEGER NOT NULL DEFAULT -1;
ALTER TABLE lux_commerce_config ADD COLUMN owner_recovery_hashes TEXT NOT NULL DEFAULT '[]';
