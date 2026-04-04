-- Create workspaces table for organizing documents (SQLite version)
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    settings TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create index on owner_id for fast workspace lookup by user
CREATE INDEX IF NOT EXISTS idx_workspaces_owner_id ON workspaces(owner_id);

-- Create index on created_at for sorting workspaces
CREATE INDEX IF NOT EXISTS idx_workspaces_created_at ON workspaces(created_at DESC);

-- Create index on name for search
CREATE INDEX IF NOT EXISTS idx_workspaces_name ON workspaces(name);
