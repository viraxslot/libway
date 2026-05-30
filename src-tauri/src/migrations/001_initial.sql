-- v1 — initial schema. Uses IF NOT EXISTS so databases created before this
-- migration system (which used CREATE TABLE IF NOT EXISTS and left
-- user_version at 0) migrate cleanly instead of erroring on existing tables.
CREATE TABLE IF NOT EXISTS repos (
    id              INTEGER PRIMARY KEY,
    owner           TEXT NOT NULL,
    name            TEXT NOT NULL,
    latest_version  TEXT,
    latest_url      TEXT,
    source_kind     TEXT,
    has_unseen      INTEGER NOT NULL DEFAULT 0,
    last_checked_at INTEGER,
    created_at      INTEGER NOT NULL,
    UNIQUE(owner, name)
);
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT
);
