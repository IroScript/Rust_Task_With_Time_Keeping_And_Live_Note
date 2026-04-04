#!/usr/bin/env python3
"""
Simple script to check SQLite database contents
"""
import sqlite3
import os

db_path = "backend/data/app.db"

if not os.path.exists(db_path):
    print(f"❌ Database not found: {db_path}")
    exit(1)

print(f"📊 Checking database: {db_path}")
print(f"📁 File size: {os.path.getsize(db_path)} bytes")
print()

conn = sqlite3.connect(db_path)
cursor = conn.cursor()

# List all tables
print("📋 Tables in database:")
cursor.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
tables = cursor.fetchall()
for table in tables:
    print(f"  - {table[0]}")
print()

# Check if our tables exist
if any('card_chunks' in str(t) for t in tables):
    print("✅ card_chunks table exists!")
    
    # Count rows
    cursor.execute("SELECT COUNT(*) FROM card_chunks")
    count = cursor.fetchone()[0]
    print(f"   Total lines stored: {count}")
    
    # Show sample data
    if count > 0:
        cursor.execute("""
            SELECT card_id, COUNT(*) as lines, 
                   MIN(line_number) as first_line, 
                   MAX(line_number) as last_line
            FROM card_chunks 
            GROUP BY card_id
        """)
        print("\n   📊 Data by card:")
        for row in cursor.fetchall():
            print(f"      Card '{row[0]}': {row[1]} lines (line {row[2]} to {row[3]})")
else:
    print("❌ card_chunks table NOT found!")
    print("   Tables need to be created. Backend should create them on startup.")

print()

# Check cards table
if any('cards' in str(t) and 'card_chunks' not in str(t) for t in tables):
    print("✅ cards table exists!")
    cursor.execute("SELECT COUNT(*) FROM cards")
    count = cursor.fetchone()[0]
    print(f"   Total cards: {count}")
else:
    print("❌ cards table NOT found!")

conn.close()
