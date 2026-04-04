-- Create revisions table for document version history (SQLite version)
CREATE TABLE IF NOT EXISTS revisions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    crdt_version INTEGER NOT NULL,
    content BLOB NOT NULL,
    content_hash BLOB NOT NULL,
    blob_ref TEXT,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    change_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create index on document_id for fast revision lookup
CREATE INDEX IF NOT EXISTS idx_revisions_document_id ON revisions(document_id);

-- Create index on user_id for fast user revision history
CREATE INDEX IF NOT EXISTS idx_revisions_user_id ON revisions(user_id);

-- Create index on created_at for revision timeline queries
CREATE INDEX IF NOT EXISTS idx_revisions_created_at ON revisions(created_at DESC);

-- Create index on crdt_version for version-based queries
CREATE INDEX IF NOT EXISTS idx_revisions_crdt_version ON revisions(document_id, crdt_version DESC);

-- Create index on content_hash for deduplication
CREATE INDEX IF NOT EXISTS idx_revisions_content_hash ON revisions(content_hash);
