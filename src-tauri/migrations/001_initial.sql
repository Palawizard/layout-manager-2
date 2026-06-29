CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY NOT NULL,
    applied_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS layouts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    minimize_unmatched_windows INTEGER NOT NULL,
    continue_on_error INTEGER NOT NULL,
    restore_previous_state_on_cancel INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS layout_actions (
    id TEXT PRIMARY KEY NOT NULL,
    layout_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    FOREIGN KEY (layout_id) REFERENCES layouts (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_layout_actions_layout_position
    ON layout_actions (layout_id, position);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
