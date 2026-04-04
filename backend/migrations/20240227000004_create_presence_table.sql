-- Create presence table for "someone is typing..." feature (SQLite version)
CREATE TABLE IF NOT EXISTS presence (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_typing INTEGER NOT NULL DEFAULT 0,
    cursor_position INTEGER,
    last_active TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(document_id, owner_id)
);

-- Create index for fast presence queries by document
CREATE INDEX IF NOT EXISTS idx_presence_document_id ON presence(document_id);

-- Create index for fast presence queries by owner
CREATE INDEX IF NOT EXISTS idx_presence_owner_id ON presence(owner_id);

-- Create index for cleanup of stale presence
CREATE INDEX IF NOT EXISTS idx_presence_last_active ON presence(last_active);
