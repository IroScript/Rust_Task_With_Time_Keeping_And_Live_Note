# Database & Table Structure Guide

## 📊 Overview

এই application এ **2 ধরনের database system** আছে:

### 1️⃣ Virtual Scrolling System (আপনার Quote App এর জন্য)
- **Tables**: `cards`, `card_chunks`
- **Purpose**: বড় text efficiently store এবং load করা
- **Location**: `backend/data/app.db`

### 2️⃣ Collaborative Document System (Future features এর জন্য)
- **Tables**: `users`, `workspaces`, `documents`, `presence`, `revisions`, `user_settings`
- **Purpose**: Multi-user document collaboration, version control
- **Location**: Same `backend/data/app.db`

---

## 🎯 Virtual Scrolling System (Active - আপনি এটা use করছেন)

### Table 1: `cards` (Master Table)

**Purpose**: প্রতিটা quote card এর metadata store করে

| Column | Type | কাজ কি? | Example |
|--------|------|---------|---------|
| `id` | TEXT (PRIMARY KEY) | Card এর unique identifier | `"quote_0"`, `"quote_23"` |
| `title` | TEXT | Card এর title/name | `"My Quote"` |
| `total_lines` | INTEGER | এই card এ কতগুলো line আছে | `8120` |
| `created_at` | TEXT | কখন create হয়েছে | `"2026-03-26 15:44:12"` |
| `updated_at` | TEXT | শেষ কখন update হয়েছে | `"2026-03-26 15:45:10"` |

**Example Data:**
```sql
id: "quote_0"
title: "quote_0"
total_lines: 8120
created_at: "2026-03-26 15:44:12"
updated_at: "2026-03-26 15:45:10"
```


---

### Table 2: `card_chunks` (Detail Table)

**Purpose**: প্রতিটা card এর actual text line-by-line store করে

| Column | Type | কাজ কি? | Example |
|--------|------|---------|---------|
| `id` | INTEGER (AUTO) | Row এর unique ID (automatic) | `1`, `2`, `3` |
| `card_id` | TEXT | কোন card এর line (foreign key) | `"quote_0"` |
| `line_number` | INTEGER | Line number (0 থেকে শুরু) | `0`, `1`, `2` |
| `line_text` | TEXT | Actual text content | `"This is line 1"` |

**Example Data:**
```sql
id: 1
card_id: "quote_0"
line_number: 0
line_text: "First line of text"

id: 2
card_id: "quote_0"
line_number: 1
line_text: "Second line of text"
```

**Index**: `idx_card_chunks_lookup` - Fast search করার জন্য `(card_id, line_number)` এর উপর

---

## 🔗 Table Relationship (কিভাবে connected?)

```
cards (Master)
  ↓
  └─── card_chunks (Details)
       (One-to-Many relationship)
```

**Explanation:**
- একটা `cards` row এর সাথে **অনেকগুলো** `card_chunks` rows connected
- `card_chunks.card_id` → `cards.id` এর সাথে link করে
- Example: `quote_0` card এর 8120 টা lines আছে মানে `card_chunks` table এ 8120 টা rows আছে যেখানে `card_id = "quote_0"`


---

## 📝 Data Flow (কখন কোন data কাজে লাগে?)

### Scenario 1: User একটা Quote Save করে (Text > 10 KB)

**Step-by-step process:**

1️⃣ **Frontend Detection** (`src/main.rs`)
   - User quote save করে
   - Frontend check করে: text size > 10 KB?
   - যদি হ্যাঁ, তাহলে virtual scrolling activate

2️⃣ **Text Split** (Frontend)
   - Text কে line-by-line split করা হয়
   - Example: 8120 lines এর text → 8120 টা separate strings

3️⃣ **API Call** (Frontend → Backend)
   - POST request: `http://localhost:3000/api/lines/save`
   - Body: 
     ```json
     {
       "card_id": "quote_0",
       "lines": ["line 1", "line 2", ..., "line 8120"]
     }
     ```

4️⃣ **Backend Processing** (`backend/src/routes/lines.rs`)
   - Backend receive করে lines array
   - Transaction শুরু করে (safety এর জন্য)

5️⃣ **Database Write - Step 1: Master Table**
   ```sql
   INSERT INTO cards (id, title, total_lines, created_at, updated_at)
   VALUES ('quote_0', 'quote_0', 8120, datetime('now'), datetime('now'))
   ON CONFLICT(id) DO UPDATE SET
     total_lines = 8120,
     updated_at = datetime('now')
   ```
   - এটা `cards` table এ master record create/update করে

6️⃣ **Database Write - Step 2: Delete Old Lines**
   ```sql
   DELETE FROM card_chunks WHERE card_id = 'quote_0'
   ```
   - পুরনো lines delete করে (যদি থাকে)

7️⃣ **Database Write - Step 3: Insert New Lines**
   ```sql
   INSERT INTO card_chunks (card_id, line_number, line_text)
   VALUES 
     ('quote_0', 0, 'First line'),
     ('quote_0', 1, 'Second line'),
     ...
     ('quote_0', 8119, 'Last line')
   ```
   - সব lines একসাথে insert করে (batch insert - fast!)

8️⃣ **Transaction Commit**
   - সব operations success হলে commit করে
   - Database এ permanently save হয়

9️⃣ **Response to Frontend**
   - Backend response পাঠায়: `200 OK`
   - Frontend log করে: "✅ Saved to backend"


---

### Scenario 2: User একটা Quote Load করে (Virtual Scrolling)

**Step-by-step process:**

1️⃣ **Frontend Request** (`src/main.rs`)
   - User scroll করে বা card open করে
   - Frontend calculate করে: কোন lines দরকার?
   - Example: Line 100 থেকে 200 পর্যন্ত

2️⃣ **API Call** (Frontend → Backend)
   - GET request: `http://localhost:3000/api/lines/quote_0?start=100&end=200`

3️⃣ **Backend Query** (`backend/src/routes/lines.rs`)
   ```sql
   SELECT line_number, line_text 
   FROM card_chunks 
   WHERE card_id = 'quote_0' 
     AND line_number >= 100 
     AND line_number < 200
   ORDER BY line_number ASC
   ```
   - শুধু দরকারি lines fetch করে (memory efficient!)

4️⃣ **Response to Frontend**
   - Backend JSON response পাঠায়:
     ```json
     {
       "card_id": "quote_0",
       "lines": [
         {"line_number": 100, "text": "Line 100 content"},
         {"line_number": 101, "text": "Line 101 content"},
         ...
       ]
     }
     ```

5️⃣ **Frontend Render**
   - Frontend শুধু এই 100 lines render করে
   - Memory তে শুধু visible lines থাকে
   - Scroll করলে নতুন lines load হয়

---

## 🔄 Operation Order (কোনটা আগে, কোনটা পরে?)

### Database Initialization (App Start এ)

```
1. Backend Start
   ↓
2. Connect to SQLite (backend/data/app.db)
   ↓
3. Apply PRAGMA optimizations
   ↓
4. Create tables (if not exists):
   - cards
   - card_chunks
   - Create index: idx_card_chunks_lookup
   ↓
5. Run migrations (create other tables):
   - users
   - workspaces
   - documents
   - presence
   - revisions
   - user_settings
   ↓
6. Backend Ready ✅
```


### Save Operation Order

```
User types text → Frontend detects size > 10KB
   ↓
Frontend splits text into lines
   ↓
Frontend sends POST /api/lines/save
   ↓
Backend starts transaction
   ↓
Backend UPSERT into cards table (master record)
   ↓
Backend DELETE old lines from card_chunks
   ↓
Backend INSERT new lines into card_chunks (batch)
   ↓
Backend commits transaction
   ↓
Backend sends 200 OK response
   ↓
Frontend shows "✅ Saved to backend"
```

### Load Operation Order

```
User scrolls or opens card
   ↓
Frontend calculates visible range (start, end)
   ↓
Frontend sends GET /api/lines/{card_id}?start=X&end=Y
   ↓
Backend queries card_chunks with range filter
   ↓
Backend returns JSON with lines
   ↓
Frontend renders only visible lines
   ↓
User sees text smoothly
```

---

## 🗂️ Collaborative Document System (Future Features - এখনো use হচ্ছে না)

এই tables গুলো future multi-user collaboration features এর জন্য তৈরি করা হয়েছে।

### Table 3: `users`

**Purpose**: User accounts store করা

| Column | কাজ কি? |
|--------|---------|
| `id` | User এর unique ID |
| `name` | User এর নাম |
| `email` | Email address (unique) |
| `country_code` | Country code |
| `company_name` | Company name |
| `created_at` | Account creation time |
| `updated_at` | Last update time |


### Table 4: `workspaces`

**Purpose**: Documents organize করার জন্য workspace/folder

| Column | কাজ কি? |
|--------|---------|
| `id` | Workspace এর unique ID |
| `owner_id` | কোন user এর workspace (→ users.id) |
| `name` | Workspace এর নাম |
| `description` | Description |
| `settings` | JSON settings |
| `created_at` | Creation time |
| `updated_at` | Last update time |

**Relationship**: `workspaces.owner_id` → `users.id`

---

### Table 5: `documents`

**Purpose**: Actual documents/files store করা

| Column | কাজ কি? |
|--------|---------|
| `id` | Document এর unique ID |
| `workspace_id` | কোন workspace এ আছে (→ workspaces.id) |
| `owner_id` | কোন user এর document (→ users.id) |
| `title` | Document title |
| `content` | Document content (BLOB) |
| `content_hash` | Content এর hash (deduplication) |
| `blob_ref` | External storage reference |
| `crdt_version` | Collaborative editing version |
| `size_bytes` | File size |
| `mime_type` | File type |
| `deleted_at` | Soft delete timestamp |
| `created_at` | Creation time |
| `updated_at` | Last update time |

**Relationships**: 
- `documents.workspace_id` → `workspaces.id`
- `documents.owner_id` → `users.id`

---

### Table 6: `presence`

**Purpose**: Real-time "who is typing" feature

| Column | কাজ কি? |
|--------|---------|
| `id` | Presence record ID |
| `document_id` | কোন document এ (→ documents.id) |
| `owner_id` | কোন user (→ users.id) |
| `is_typing` | Currently typing? (0/1) |
| `cursor_position` | Cursor position |
| `last_active` | Last activity time |

**Relationships**: 
- `presence.document_id` → `documents.id`
- `presence.owner_id` → `users.id`


### Table 7: `revisions`

**Purpose**: Document version history (undo/redo, time travel)

| Column | কাজ কি? |
|--------|---------|
| `id` | Revision ID |
| `document_id` | কোন document এর version (→ documents.id) |
| `user_id` | কে change করেছে (→ users.id) |
| `crdt_version` | Version number |
| `content` | Version এর content (BLOB) |
| `content_hash` | Content hash |
| `blob_ref` | External storage reference |
| `size_bytes` | Version size |
| `mime_type` | File type |
| `change_summary` | What changed? |
| `created_at` | Version creation time |

**Relationships**: 
- `revisions.document_id` → `documents.id`
- `revisions.user_id` → `users.id`

---

### Table 8: `user_settings`

**Purpose**: User preferences (theme, text style, etc.)

| Column | কাজ কি? |
|--------|---------|
| `id` | Settings record ID |
| `user_id` | কোন user এর settings (→ users.id) |
| `settings_data` | JSON settings data |
| `created_at` | Creation time |
| `updated_at` | Last update time |

**Relationship**: `user_settings.user_id` → `users.id` (One-to-One)

---

## 🔗 Complete Relationship Diagram

```
users (Master)
  ↓
  ├─── workspaces (One user → Many workspaces)
  │      ↓
  │      └─── documents (One workspace → Many documents)
  │             ↓
  │             ├─── presence (One document → Many presence records)
  │             └─── revisions (One document → Many versions)
  │
  ├─── documents (One user → Many documents)
  │      ↓
  │      ├─── presence (One document → Many presence records)
  │      └─── revisions (One document → Many versions)
  │
  └─── user_settings (One user → One settings)

cards (Independent - Virtual Scrolling)
  ↓
  └─── card_chunks (One card → Many lines)
```


---

## 💡 Key Concepts

### 1. Foreign Key (কিভাবে tables connected?)

**Example:**
```sql
card_chunks.card_id → cards.id
```
- `card_chunks` table এর `card_id` column টা `cards` table এর `id` column এর সাথে link করে
- এটাকে বলে "Foreign Key Relationship"
- Meaning: প্রতিটা `card_chunks` row অবশ্যই একটা valid `cards` row এর সাথে connected

### 2. One-to-Many Relationship

**Example:**
```
cards (1) ←→ card_chunks (Many)
```
- একটা card এর অনেকগুলো chunks/lines থাকতে পারে
- কিন্তু একটা chunk শুধুমাত্র একটা card এর সাথে belong করে

### 3. Index (কেন দরকার?)

**Example:**
```sql
CREATE INDEX idx_card_chunks_lookup ON card_chunks(card_id, line_number)
```

**Without Index:**
```
Query: "Find line 5000 of quote_0"
Database: Checks ALL 8120 rows one by one 😰
Time: Slow (100ms+)
```

**With Index:**
```
Query: "Find line 5000 of quote_0"
Database: Uses index to jump directly to line 5000 🚀
Time: Fast (1-2ms)
```

### 4. Transaction (কেন safe?)

**Without Transaction:**
```
Step 1: Insert into cards ✅
Step 2: Delete old chunks ✅
Step 3: Insert new chunks ❌ (ERROR!)
Result: Data corrupted! 😱
```

**With Transaction:**
```
BEGIN TRANSACTION
  Step 1: Insert into cards ✅
  Step 2: Delete old chunks ✅
  Step 3: Insert new chunks ❌ (ERROR!)
ROLLBACK (undo everything)
Result: Data safe! Original data intact 🛡️
```


---

## 🎯 আপনার Current Usage

### Active Tables (এখন use হচ্ছে):
1. ✅ **cards** - Quote metadata
2. ✅ **card_chunks** - Quote text lines

### Inactive Tables (future এর জন্য ready):
3. ⏸️ **users** - User accounts
4. ⏸️ **workspaces** - Document folders
5. ⏸️ **documents** - Collaborative documents
6. ⏸️ **presence** - Real-time typing status
7. ⏸️ **revisions** - Version history
8. ⏸️ **user_settings** - User preferences

---

## 📊 Real Example from Your App

### Backend Log Analysis:
```
2026-03-26T15:44:12.490132Z  INFO backend::routes::lines: Batch inserted lines card_id=quote_1 count=3
2026-03-26T15:44:35.457259Z  INFO backend::routes::lines: Batch inserted lines card_id=quote_23 count=8120
2026-03-26T15:44:51.080769Z  INFO backend::routes::lines: Batch inserted lines card_id=quote_0 count=8120
```

**What happened?**

1. **quote_1**: 3 lines saved
   - `cards` table: 1 row (id="quote_1", total_lines=3)
   - `card_chunks` table: 3 rows (line 0, 1, 2)

2. **quote_23**: 8120 lines saved
   - `cards` table: 1 row (id="quote_23", total_lines=8120)
   - `card_chunks` table: 8120 rows (line 0 to 8119)

3. **quote_0**: 8120 lines saved
   - `cards` table: 1 row (id="quote_0", total_lines=8120)
   - `card_chunks` table: 8120 rows (line 0 to 8119)

**Total Database Size:**
- `cards`: 3 rows
- `card_chunks`: 3 + 8120 + 8120 = 16,243 rows
- Database file: `backend/data/app.db` (159,744 bytes ≈ 156 KB)


---

## 🔍 How to Verify Data

### Method 1: Using Python Script
```bash
python check_db.py
```

### Method 2: Using PowerShell Script
```powershell
.\check_db.ps1
```

### Method 3: Using DB Browser for SQLite
1. Download: https://sqlitebrowser.org/
2. Open: `backend/data/app.db`
3. Browse Data tab
4. Select table: `cards` or `card_chunks`
5. See all data visually

### Method 4: Direct SQL Query
```bash
cd backend
sqlite3 data/app.db "SELECT * FROM cards;"
sqlite3 data/app.db "SELECT COUNT(*) FROM card_chunks WHERE card_id='quote_0';"
```

---

## 🚀 Performance Optimizations

### PRAGMA Settings (Applied at startup)

| Setting | Value | কেন? |
|---------|-------|------|
| `journal_mode` | WAL | Write-Ahead Logging - faster writes |
| `synchronous` | NORMAL | Balance between speed & safety |
| `cache_size` | -10000 | 10 MB cache (negative = KB) |
| `temp_store` | MEMORY | Temp data in RAM (faster) |
| `mmap_size` | 0 | No memory mapping (500 MB RAM constraint) |
| `page_size` | 4096 | 4 KB pages (standard) |
| `foreign_keys` | ON | Enforce relationships |

### Why These Settings?

**500 MB RAM Constraint:**
- Single connection pool (max_connections=1)
- Small cache (10 MB only)
- No memory mapping (mmap_size=0)
- Temp data in memory (faster than disk)

**Result:**
- Fast enough for your use case
- Memory efficient
- Safe data storage


---

## 🎓 Summary

### Virtual Scrolling System Flow:

```
User Types Large Text (> 10 KB)
         ↓
Frontend Splits into Lines
         ↓
POST /api/lines/save
         ↓
Backend Transaction Starts
         ↓
┌─────────────────────────┐
│  cards table            │
│  - Store metadata       │
│  - total_lines count    │
└─────────────────────────┘
         ↓
┌─────────────────────────┐
│  card_chunks table      │
│  - Store each line      │
│  - line_number indexed  │
└─────────────────────────┘
         ↓
Transaction Commits
         ↓
Response: 200 OK
         ↓
Frontend: "✅ Saved to backend"
```

### Key Takeaways:

1. **2 Tables Work Together**: `cards` (master) + `card_chunks` (details)
2. **Line-by-Line Storage**: Each line = 1 row in `card_chunks`
3. **Fast Retrieval**: Index on `(card_id, line_number)` makes queries instant
4. **Safe Operations**: Transactions ensure data integrity
5. **Memory Efficient**: Load only visible lines, not entire text
6. **Scalable**: Can handle millions of lines without performance issues

### আপনার App এ:
- ✅ Virtual scrolling active
- ✅ Data saving to database
- ✅ Backend logs confirm success
- ✅ 16,243 lines stored across 3 cards
- ✅ Database size: 156 KB (very efficient!)

---

**Questions?** এই guide এ যদি কোনো কিছু unclear থাকে, আমাকে জানান! 😊
