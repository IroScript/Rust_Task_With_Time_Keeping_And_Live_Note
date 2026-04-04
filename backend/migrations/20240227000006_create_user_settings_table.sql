-- Create user_settings table for storing theme and text style settings (SQLite version)
CREATE TABLE IF NOT EXISTS user_settings (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    settings_data TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Ensure one settings record per user
    UNIQUE(user_id)
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id);

-- Trigger for updated_at (SQLite version)
CREATE TRIGGER IF NOT EXISTS update_user_settings_updated_at 
    AFTER UPDATE ON user_settings 
    FOR EACH ROW 
BEGIN
    UPDATE user_settings SET updated_at = datetime('now') WHERE id = NEW.id;
END;
