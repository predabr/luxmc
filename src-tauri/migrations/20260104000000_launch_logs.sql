CREATE TABLE IF NOT EXISTS launch_logs (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	profile_id TEXT,
	version_id TEXT NOT NULL,
	started_at TEXT NOT NULL,
	ended_at TEXT,
	exit_code INTEGER,
	summary TEXT,
	error_classification TEXT
);

CREATE INDEX IF NOT EXISTS idx_launch_logs_started ON launch_logs (started_at DESC);
CREATE INDEX IF NOT EXISTS idx_launch_logs_profile ON launch_logs (profile_id);

CREATE TABLE IF NOT EXISTS launch_log_lines (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	log_id INTEGER NOT NULL,
	stream TEXT NOT NULL,
	level TEXT NOT NULL,
	message TEXT NOT NULL,
	ts TEXT NOT NULL,
	FOREIGN KEY (log_id) REFERENCES launch_logs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_launch_log_lines_log ON launch_log_lines (log_id);
