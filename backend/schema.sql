-- ============================================================
-- Pure Rust Virtual Scrolling System - Database Schema
-- ============================================================
-- This schema supports storing billions of lines per card
-- with efficient O(log n) lookup via B-Tree indexing
-- ============================================================

-- ============================================================
-- TABLE 1: cards
-- ============================================================
-- Stores card metadata including total line count
CREATE TABLE IF NOT EXISTS cards (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    total_lines     INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- TABLE 2: card_chunks
-- ============================================================
-- Stores individual lines of text for each card
-- Each row represents ONE line of text
-- Supports billions of rows with efficient indexing
CREATE TABLE IF NOT EXISTS card_chunks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id         INTEGER NOT NULL,
    line_number     INTEGER NOT NULL,
    line_text       TEXT NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

-- ============================================================
-- CRITICAL INDEX: Composite B-Tree Index
-- ============================================================
-- This index enables O(log n) lookup for any line range
-- Even with billions of rows, lookup is near-instant
CREATE INDEX IF NOT EXISTS idx_card_chunks_lookup 
ON card_chunks(card_id, line_number);

-- ============================================================
-- PRAGMA Optimizations for 500 MB RAM constraint
-- ============================================================
-- These settings MUST be applied on every connection open

-- Write-Ahead Logging: Allows concurrent reads during writes
PRAGMA journal_mode = WAL;

-- Balance between safety and speed
PRAGMA synchronous = NORMAL;

-- Limit cache to ~10 MB (negative value = KB)
PRAGMA cache_size = -10000;

-- Store temporary tables in memory (small ops only)
PRAGMA temp_store = MEMORY;

-- Disable memory-mapped I/O (saves RAM on 500MB system)
PRAGMA mmap_size = 0;

-- Match OS page size for optimal I/O
PRAGMA page_size = 4096;

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- ============================================================
-- Sample Data for Testing
-- ============================================================
-- Insert a test card
INSERT OR IGNORE INTO cards (id, title, total_lines) 
VALUES (1, 'Test Card - Large Text', 0);

-- ============================================================
-- Useful Queries for Debugging
-- ============================================================

-- Check total cards
-- SELECT COUNT(*) FROM cards;

-- Check total lines across all cards
-- SELECT SUM(total_lines) FROM cards;

-- Check lines for a specific card
-- SELECT COUNT(*) FROM card_chunks WHERE card_id = 1;

-- Fetch a range of lines (virtual scrolling query)
-- SELECT line_number, line_text 
-- FROM card_chunks 
-- WHERE card_id = 1 AND line_number >= 500000 
-- ORDER BY line_number ASC 
-- LIMIT 50;

-- Check index usage (should use idx_card_chunks_lookup)
-- EXPLAIN QUERY PLAN 
-- SELECT line_number, line_text 
-- FROM card_chunks 
-- WHERE card_id = 1 AND line_number >= 500000 
-- ORDER BY line_number ASC 
-- LIMIT 50;

-- ============================================================
-- Performance Notes
-- ============================================================
-- 1. B-Tree index allows binary search through billions of rows
-- 2. WAL mode enables concurrent reads during data ingestion
-- 3. Prepared statements (prepare_cached) avoid SQL re-parsing
-- 4. Batch inserts (10,000 rows per transaction) maximize throughput
-- 5. Page size 4096 matches typical OS/filesystem block size
-- ============================================================
