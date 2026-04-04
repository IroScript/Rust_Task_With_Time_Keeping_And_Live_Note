//! CLI tool for ingesting large text files into card_chunks table
//!
//! Usage: cli <db_path> <card_id> <file_path>
//! Example: cli backend/data/app.db 1 large_text.txt
//!
//! This tool reads a text file line-by-line and inserts each line
//! into the card_chunks table with proper line numbering.
//! Memory usage stays flat regardless of file size.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rusqlite::{params, Connection};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const BATCH_SIZE: usize = 10_000;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 4 {
        eprintln!("Usage: {} <db_path> <card_id> <file_path>", args[0]);
        eprintln!("Example: {} backend/data/app.db 1 large_text.txt", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    let card_id: i64 = args[2]
        .parse()
        .context("card_id must be a valid integer")?;
    let file_path = &args[3];

    println!("📂 Opening database: {}", db_path);
    println!("📄 Reading file: {}", file_path);
    println!("🎯 Target card ID: {}", card_id);
    println!();

    ingest_file(db_path, card_id, file_path)?;

    println!("\n✅ Ingestion complete!");
    Ok(())
}

fn ingest_file(db_path: &str, card_id: i64, file_path: &str) -> Result<()> {
    // Open database connection
    let mut conn = Connection::open(db_path)
        .context("Failed to open database")?;

    // Apply PRAGMA optimizations
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -10000;
         PRAGMA temp_store = MEMORY;"
    )?;

    // Open file with BufReader (8 KB internal buffer)
    let file = File::open(file_path)
        .context("Failed to open input file")?;
    let reader = BufReader::new(file);

    // Count total lines for progress bar
    println!("📊 Counting lines...");
    let total_lines = count_lines(file_path)?;
    println!("📏 Total lines: {}", total_lines);
    println!();

    // Create progress bar
    let pb = ProgressBar::new(total_lines as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Prepare INSERT statement
    let mut stmt = conn.prepare_cached(
        "INSERT INTO card_chunks (card_id, line_number, line_text) VALUES (?1, ?2, ?3)"
    )?;

    let mut line_number: i64 = 0;
    let mut batch_count = 0;
    let mut total_inserted = 0;

    // Begin first transaction
    let tx = conn.transaction()?;

    for line_result in reader.lines() {
        let line_text = line_result.context("Failed to read line")?;

        // Insert line
        stmt.execute(params![card_id, line_number, line_text])?;

        line_number += 1;
        batch_count += 1;
        total_inserted += 1;

        // Update progress bar
        if total_inserted % 1000 == 0 {
            pb.set_position(total_inserted as u64);
        }

        // Commit transaction every BATCH_SIZE lines
        if batch_count >= BATCH_SIZE {
            drop(stmt); // Drop statement before committing
            tx.commit()?;

            // Start new transaction
            let tx = conn.transaction()?;
            stmt = conn.prepare_cached(
                "INSERT INTO card_chunks (card_id, line_number, line_text) VALUES (?1, ?2, ?3)"
            )?;

            batch_count = 0;
        }
    }

    // Commit remaining lines
    if batch_count > 0 {
        drop(stmt);
        tx.commit()?;
    }

    pb.finish_with_message("✅ All lines inserted");

    // Update total_lines in cards table
    println!("\n📝 Updating card metadata...");
    conn.execute(
        "UPDATE cards SET total_lines = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![line_number, card_id],
    )?;

    println!("✅ Card metadata updated");
    println!("📊 Total lines inserted: {}", total_inserted);

    Ok(())
}

fn count_lines(file_path: &str) -> Result<usize> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}
