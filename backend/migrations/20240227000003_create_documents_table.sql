-- Create documents table (SQLite version)
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content BLOB,
    content_hash BLOB,
    blob_ref TEXT,
    crdt_version INTEGER NOT NULL DEFAULT 0,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create index on workspace_id for fast document queries by workspace
CREATE INDEX IF NOT EXISTS idx_documents_workspace_id ON documents(workspace_id);

-- Create index on owner_id for fast user document queries
CREATE INDEX IF NOT EXISTS idx_documents_owner_id ON documents(owner_id);

-- Create composite index for workspace + owner queries
CREATE INDEX IF NOT EXISTS idx_documents_workspace_owner ON documents(workspace_id, owner_id);

-- Create index on updated_at for sorting
CREATE INDEX IF NOT EXISTS idx_documents_updated_at ON documents(updated_at DESC);

-- Create index on created_at for document sorting
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at DESC);

-- Create index on content_hash for deduplication
CREATE INDEX IF NOT EXISTS idx_documents_content_hash ON documents(content_hash);
