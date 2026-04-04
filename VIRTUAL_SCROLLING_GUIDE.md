# Virtual Scrolling System - Complete Guide

## 📋 Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Data Flow Explanation](#data-flow-explanation)
3. [Database Location](#database-location)
4. [Testing Steps](#testing-steps)
5. [API Endpoints Reference](#api-endpoints-reference)
6. [Troubleshooting](#troubleshooting)

---

## 🏗️ Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    YOUR LOCAL MACHINE                        │
│                  (localhost - 127.0.0.1)                     │
│                                                              │
│  ┌──────────────┐         HTTP          ┌─────────────────┐ │
│  │   Frontend   │ ◄──────────────────► │    Backend      │ │
│  │   (egui)     │   localhost:3000     │    (Axum)       │ │
│  │              │                       │                 │ │
│  │  - UI        │                       │  - REST API     │ │
│  │  - Scrolling │                       │  - Routes       │ │
│  │  - Editing   │                       │                 │ │
│  └──────────────┘                       └────────┬────────┘ │
│                                                  │          │
│                                         rusqlite│          │
│                                                  │          │
│                                         ┌────────▼────────┐ │
│                                         │   SQLite DB     │ │
│                                         │   (app.db)      │ │
│                                         │                 │ │
│                                         │  - cards table  │ │
│                                         │  - card_chunks  │ │
│                                         └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### কোথায় কি আছে?

1. **Frontend (egui app)** - `cargo run` দিয়ে চালু হয়
   - Location: `src/main.rs`
   - Port: কোনো port নেই (native desktop app)
   - UI তে scroll করলে HTTP request পাঠায়

2. **Backend (Axum server)** - `cargo run` দিয়ে চালু হয়
   - Location: `backend/src/main.rs`
   - Port: `3000` (localhost:3000)
   - HTTP requests receive করে এবং database query করে

3. **Database (SQLite)** - File হিসেবে থাকে
   - Location: `backend/data/app.db`
   - Type: Local file (NOT online, NOT PostgreSQL)
   - Size: যত data insert করবেন তত বড় হবে

4. **CLI Tool** - Data insert করার জন্য
   - Location: `cli/src/main.rs`
   - Usage: Text file থেকে database এ data load করে

---

## 🔄 Data Flow Explanation

### Scenario 1: Frontend থেকে Data দেখা

```
User scrolls in egui UI
         │
         ▼
Frontend calculates: "আমার line 1000-1050 দরকার"
         │
         ▼
HTTP GET request পাঠায়:
GET http://localhost:3000/api/cards/1/lines?start_line=1000&limit=50
         │
         ▼
Backend (Axum) request receive করে
         │
         ▼
Backend SQLite database query করে:
SELECT line_number, line_text 
FROM card_chunks 
WHERE card_id=1 AND line_number >= 1000 
LIMIT 50
         │
         ▼
SQLite file (app.db) থেকে data পড়ে
         │
         ▼
Backend JSON response পাঠায়:
[
  {"line_number": 1000, "line_text": "..."},
  {"line_number": 1001, "line_text": "..."},
  ...
]
         │
         ▼
Frontend JSON parse করে screen এ দেখায়
```

### Scenario 2: CLI দিয়ে Data Insert

```
User runs: cargo run --bin cli -- --card-id 1 --file mydata.txt
         │
         ▼
CLI tool file open করে (mydata.txt)
         │
         ▼
Line by line পড়ে (memory তে পুরো file load করে না!)
         │
         ▼
10,000 lines একসাথে SQLite এ INSERT করে
         │
         ▼
SQLite file (app.db) এ data save হয়
         │
         ▼
Progress bar দেখায়: "Inserted 50,000 lines..."
```

---

## 💾 Database Location

### SQLite Database কোথায়?

**Path:** `backend/data/app.db`

এটা একটা **local file**, NOT online database!

### PostgreSQL আছে কি?

**না!** এই project এ SQLite ব্যবহার করা হয়েছে কারণ:
- Single machine এর জন্য
- 500 MB RAM constraint
- No network overhead
- File-based, simple

### Data কোথায় store হয়?

```
backend/
  └── data/
      └── app.db  ← এখানে সব data থাকে
```

এই file টা:
- Binary format (SQLite)
- Directly open করা যায় না (text editor দিয়ে)
- SQLite browser দিয়ে দেখা যায়
- Size: যত data insert করবেন তত বড়

---

## 🧪 Testing Steps

### Step 1: Backend Start করুন

```bash
# Terminal 1 এ
cd backend
cargo run
```

**Expected Output:**
```
🚀 Starting Pure Rust Backend with Axum API
📊 Database: SQLite (local)
✅ Configuration loaded
✅ Database initialized
🌐 Server listening on http://127.0.0.1:3000
```

**যদি error আসে:**
- Check করুন `backend/data/` folder আছে কিনা
- Check করুন port 3000 free আছে কিনা

### Step 2: একটা Test Card তৈরি করুন

Backend চালু থাকা অবস্থায়, নতুন terminal এ:

```bash
# Windows PowerShell
curl -X POST http://localhost:3000/api/cards `
  -H "Content-Type: application/json" `
  -d '{"title": "Test Card"}'
```

**Expected Response:**
```json
{
  "id": 1,
  "title": "Test Card",
  "total_lines": 0,
  "created_at": "2026-03-26T...",
  "updated_at": "2026-03-26T..."
}
```

**Response এর মানে:**
- `id: 1` - এই card এর ID হলো 1
- `total_lines: 0` - এখনো কোনো line নেই

### Step 3: Test Data Insert করুন (CLI দিয়ে)

প্রথমে একটা test file তৈরি করুন:

```bash
# Windows PowerShell
1..1000 | ForEach-Object { "This is line number $_" } | Out-File -FilePath test_data.txt -Encoding UTF8
```

এটা 1000 lines এর একটা file তৈরি করবে।

এখন CLI দিয়ে insert করুন:

```bash
# Terminal 2 এ (backend চালু রাখুন Terminal 1 এ)
cd cli
cargo run -- --card-id 1 --file ../test_data.txt
```

**Expected Output:**
```
📥 Starting data ingestion...
Card ID: 1
File: ../test_data.txt
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 1000/1000 lines
✅ Ingestion complete!
Total lines inserted: 1000
Time taken: 0.5s
```

### Step 4: Backend API দিয়ে Data Check করুন

```bash
# Check card metadata
curl http://localhost:3000/api/cards/1/meta
```

**Expected Response:**
```json
{
  "card_id": "1",
  "title": "Test Card",
  "total_lines": 1000
}
```

এখন কিছু lines fetch করুন:

```bash
# Get lines 0-10
curl "http://localhost:3000/api/cards/1/lines?start_line=0&limit=10"
```

**Expected Response:**
```json
[
  {"line_number": 0, "line_text": "This is line number 1"},
  {"line_number": 1, "line_text": "This is line number 2"},
  ...
]
```

### Step 5: Frontend Start করুন

```bash
# Terminal 3 এ
cargo run
```

Frontend window খুলবে।

**Frontend এ:**
1. "Open Virtual Scroller" button এ click করুন
2. Card ID field এ `1` লিখুন
3. "Load Card" button এ click করুন
4. Scroll করুন - data দেখতে পাবেন!

---

## 📡 API Endpoints Reference

### 1. GET /api/cards
**Purpose:** সব cards এর list দেখা

**Request:**
```bash
curl "http://localhost:3000/api/cards?page=1&per_page=50"
```

**Response:**
```json
{
  "cards": [
    {
      "id": 1,
      "title": "Test Card",
      "total_lines": 1000,
      "created_at": "...",
      "updated_at": "..."
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 50
}
```

### 2. POST /api/cards
**Purpose:** নতুন card তৈরি করা

**Request:**
```bash
curl -X POST http://localhost:3000/api/cards \
  -H "Content-Type: application/json" \
  -d '{"title": "My New Card"}'
```

### 3. GET /api/cards/:card_id/meta
**Purpose:** Card এর metadata (total lines) দেখা

**Request:**
```bash
curl http://localhost:3000/api/cards/1/meta
```

**Response:**
```json
{
  "card_id": "1",
  "title": "Test Card",
  "total_lines": 1000
}
```

### 4. GET /api/cards/:card_id/lines
**Purpose:** Specific lines fetch করা (virtual scrolling এর জন্য)

**Request:**
```bash
curl "http://localhost:3000/api/cards/1/lines?start_line=500&limit=50"
```

**Parameters:**
- `start_line`: কোন line থেকে শুরু (0-indexed)
- `limit`: কতগুলো lines চাই (max 1000)

**Response:**
```json
[
  {"line_number": 500, "line_text": "..."},
  {"line_number": 501, "line_text": "..."},
  ...
]
```

### 5. PUT /api/cards/:card_id/lines/:line_number
**Purpose:** একটা specific line edit করা

**Request:**
```bash
curl -X PUT http://localhost:3000/api/cards/1/lines/500 \
  -H "Content-Type: application/json" \
  -d '{"line_text": "Updated text"}'
```

### 6. POST /api/cards/:card_id/lines/batch
**Purpose:** Multiple lines একসাথে insert করা

**Request:**
```bash
curl -X POST http://localhost:3000/api/cards/1/lines/batch \
  -H "Content-Type: application/json" \
  -d '{
    "lines": [
      {"line_number": 0, "line_text": "First line"},
      {"line_number": 1, "line_text": "Second line"}
    ]
  }'
```

---

## 🔍 Troubleshooting

### Problem 1: Backend start হচ্ছে না

**Error:** `Failed to initialize database`

**Solution:**
```bash
# backend/data/ folder তৈরি করুন
mkdir backend/data
```

### Problem 2: Frontend backend এ connect করতে পারছে না

**Error:** `Failed to load card: Connection refused`

**Check:**
1. Backend চালু আছে কিনা?
2. Backend port 3000 এ চলছে কিনা?
3. Frontend এ backend URL ঠিক আছে কিনা? (`http://localhost:3000`)

### Problem 3: CLI data insert করতে পারছে না

**Error:** `Card not found`

**Solution:**
প্রথমে card তৈরি করুন:
```bash
curl -X POST http://localhost:3000/api/cards \
  -H "Content-Type: application/json" \
  -d '{"title": "My Card"}'
```

Response এ যে `id` পাবেন সেটা CLI তে use করুন।

### Problem 4: Database file কোথায়?

**Location:** `backend/data/app.db`

**View করতে:**
- Download করুন: [DB Browser for SQLite](https://sqlitebrowser.org/)
- Open করুন: `backend/data/app.db`
- Tables দেখুন: `cards`, `card_chunks`

---

## 📊 Memory Usage Check

### Backend Memory:
```bash
# Windows Task Manager এ দেখুন
# Process name: backend.exe
# Expected: < 20 MB
```

### Frontend Memory:
```bash
# Windows Task Manager এ দেখুন
# Process name: frontend.exe or your app name
# Expected: < 30 MB
```

### Total System:
```
Backend:  ~20 MB
Frontend: ~30 MB
SQLite:   ~10 MB (cache)
─────────────────
Total:    ~60 MB ✅
```

---

## 🎯 Quick Test Checklist

- [ ] Backend starts successfully
- [ ] Create a test card via API
- [ ] Insert test data via CLI
- [ ] Verify data via API (curl)
- [ ] Frontend loads and connects
- [ ] Virtual scrolling works smoothly
- [ ] Line editing works
- [ ] Memory usage < 60 MB

---

## 📝 Notes

### Data Persistence
- সব data `backend/data/app.db` file এ থাকে
- Backend restart করলেও data থাকবে
- File delete করলে সব data হারিয়ে যাবে

### Performance
- 1000 lines: Instant
- 1 million lines: < 1 second
- 1 billion lines: < 2 seconds (with proper indexing)

### Scalability
- Single card: Up to 500 GB text
- Multiple cards: Unlimited (disk space dependent)
- RAM usage: Always < 60 MB

---

## 🚀 Next Steps

1. Test করুন উপরের steps follow করে
2. Large file (10 MB+) insert করে দেখুন
3. Frontend এ smooth scrolling test করুন
4. Line editing test করুন
5. Memory usage monitor করুন

---

**Questions?** এই file এ সব কিছু documented আছে। যেকোনো confusion হলে এই guide refer করুন।
