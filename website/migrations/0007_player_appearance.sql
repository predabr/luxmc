CREATE TABLE IF NOT EXISTS lux_appearance (
  account_id TEXT PRIMARY KEY REFERENCES lux_accounts(id) ON DELETE CASCADE,
  skin TEXT NOT NULL,
  cape TEXT,
  model TEXT NOT NULL CHECK(model IN ('classic', 'slim')),
  updated_at INTEGER NOT NULL
);
