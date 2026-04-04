CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    total_lines INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS card_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    line_text TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_card_chunks_lookup 
ON card_chunks(card_id, line_number);
