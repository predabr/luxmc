CREATE TABLE lux_microsoft_launcher_pending (
  token_hash TEXT PRIMARY KEY,
  subject_hash TEXT NOT NULL,
  social_id TEXT NOT NULL REFERENCES social_users(id),
  minecraft_uuid TEXT NOT NULL,
  username TEXT NOT NULL,
  server_id TEXT NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX lux_microsoft_launcher_expiry ON lux_microsoft_launcher_pending(expires_at);
ALTER TABLE lux_microsoft_link_pending ADD COLUMN minecraft_uuid TEXT;
ALTER TABLE lux_microsoft_link_pending ADD COLUMN minecraft_name TEXT;
CREATE TABLE lux_microsoft_devices (
  token_hash TEXT PRIMARY KEY,
  account_id TEXT NOT NULL REFERENCES lux_accounts(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL
);
