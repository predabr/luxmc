-- accounts: stored Microsoft/Xbox auth results
CREATE TABLE IF NOT EXISTS accounts (
	id TEXT PRIMARY KEY,
	username TEXT NOT NULL,
	uuid TEXT NOT NULL UNIQUE,
	refresh_token TEXT NOT NULL,
	access_token TEXT,
	expires_at TEXT,
	created_at TEXT NOT NULL,
	updated_at TEXT NOT NULL
);

-- profiles: independent game instances
CREATE TABLE IF NOT EXISTS profiles (
	id TEXT PRIMARY KEY,
	name TEXT NOT NULL,
	icon TEXT NOT NULL DEFAULT 'grass_block',
	mc_version TEXT NOT NULL,
	loader TEXT NOT NULL,
	loader_version TEXT,
	java_path TEXT,
	jvm_args TEXT,
	resolution_w INTEGER,
	resolution_h INTEGER,
	fullscreen INTEGER NOT NULL DEFAULT 0,
	game_dir TEXT NOT NULL,
	created_at TEXT NOT NULL,
	updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_profiles_mc_version ON profiles (mc_version);
CREATE INDEX IF NOT EXISTS idx_profiles_loader ON profiles (loader);

-- mc_versions: cached version manifest
CREATE TABLE IF NOT EXISTS mc_versions (
	id TEXT PRIMARY KEY,
	type TEXT NOT NULL,
	url TEXT NOT NULL,
	time TEXT NOT NULL,
	release_time TEXT NOT NULL,
	fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- mods: per-profile mod manifest
CREATE TABLE IF NOT EXISTS mods (
	profile_id TEXT NOT NULL,
	project_id TEXT NOT NULL,
	version_id TEXT NOT NULL,
	file_name TEXT NOT NULL,
	sha1 TEXT NOT NULL,
	source TEXT NOT NULL,
	installed_at TEXT NOT NULL,
	PRIMARY KEY (profile_id, project_id),
	FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

-- settings: key-value
CREATE TABLE IF NOT EXISTS app_settings (
	key TEXT PRIMARY KEY,
	value TEXT NOT NULL
);
