# Virtual Scrolling System - Questions & Answers

এই file এ আপনার সব প্রশ্ন এবং তার উত্তর থাকবে।

---

## ❓ Question 1: HTTP মানে কি online না?

**Your Question:**
> UI তে scroll করলে HTTP request পাঠায়। কিন্তু HTTP তো online ছাড়া হওয়ার কথা না?

**Answer:**

**না! HTTP শুধু online এর জন্য না।**

HTTP = একটা communication protocol (যোগাযোগের নিয়ম)

### HTTP দুই জায়গায় ব্যবহার হয়:

#### 1. Online (Internet):
```
Your Browser → google.com (Internet এর মাধ্যমে)
```

#### 2. Localhost (Your Own Computer):
```
Frontend → localhost:3000 → Backend (একই computer এ)
```

### এই project এ কি হচ্ছে?

```
┌─────────────────────────────────────────┐
│        Your Computer (Offline)          │
│                                         │
│  Frontend ──HTTP──→ Backend             │
│  (Port নেই)      (Port 3000)           │
│                                         │
│  ↑                                      │
│  └─── Same machine, NO internet! ───────┘
```

**Example:**
- আপনি যখন `http://localhost:3000` লিখেন
- `localhost` = আপনার নিজের computer
- `127.0.0.1` = আপনার নিজের computer এর IP address
- Internet connection লাগে না!

### Test করে দেখুন:

1. Internet disconnect করুন (WiFi off)
2. Backend চালু করুন: `cargo run` (backend folder এ)
3. Browser এ যান: `http://localhost:3000/health`
4. দেখবেন কাজ করছে! কারণ সব আপনার computer এই হচ্ছে।

---

## ❓ Question 2: Data কোথায় store হয়?

**Your Question:**
> Frontend থেকে data কোথায় কোথায় যাচ্ছে বা আসছে? Local, JSON, online PostgreSQL আছে?

**Answer:**

### Data Flow (Step by Step):

```
1. User scrolls in Frontend
         ↓
2. Frontend sends HTTP request to Backend
   URL: http://localhost:3000/api/cards/1/lines?start_line=100&limit=50
         ↓
3. Backend receives request
         ↓
4. Backend queries SQLite database
   File location: backend/data/app.db
         ↓
5. SQLite returns data
         ↓
6. Backend sends JSON response to Frontend
         ↓
7. Frontend displays data on screen
```

### Data Storage Location:

**Physical File:** `backend/data/app.db`

```
Your Project Folder/
├── backend/
│   └── data/
│       └── app.db  ← এখানে সব data আছে!
```

### এটা কি?

- ✅ **Local file** (আপনার hard disk এ)
- ✅ **SQLite database** (file-based database)
- ❌ **NOT online**
- ❌ **NOT PostgreSQL** (PostgreSQL একটা আলাদা database system)
- ❌ **NOT JSON file** (SQLite binary format)

### JSON কোথায় ব্যবহার হয়?

JSON শুধু **communication** এর জন্য:

```
Backend → Frontend
{
  "line_number": 100,
  "line_text": "Hello World"
}
```

কিন্তু **storage** SQLite file এ (app.db)।

---

## ❓ Question 3: PostgreSQL আছে কি?

**Your Question:**
> Online PostgreSQL আছে?

**Answer:**

**না! এই project এ PostgreSQL নেই।**

### কেন SQLite ব্যবহার করা হয়েছে?

1. **Single machine** এর জন্য (আপনার computer)
2. **500 MB RAM** constraint
3. **No server setup** দরকার নেই
4. **File-based** - সহজ
5. **Fast** - local file access

### PostgreSQL vs SQLite:

| Feature | PostgreSQL | SQLite (আমাদের) |
|---------|-----------|-----------------|
| Location | Server (online/offline) | File (local) |
| Setup | Complex | Simple |
| RAM | High | Low |
| Use case | Multi-user | Single-user |

### আপনার project এ:

```
Database: SQLite
Location: backend/data/app.db
Type: Local file
Internet: NOT needed
```

---

## ❓ Question 4: কিভাবে test করব?

**Your Question:**
> এখন কি test করব? কিভাবে করব?

**Answer:**

### Step-by-Step Testing Guide:

#### Step 1: Backend Start করুন

```bash
# Terminal 1 খুলুন
cd backend
cargo run
```

**দেখবেন:**
```
🚀 Starting Pure Rust Backend
🌐 Server listening on http://127.0.0.1:3000
```

<details>
<summary><b>📖 এর মানে কি? (Click to expand)</b></summary>

### এই message এর মানে:

#### 1. `http://127.0.0.1:3000` কি?

**Breaking down:**
- `http://` = Protocol (communication এর নিয়ম)
- `127.0.0.1` = আপনার নিজের computer এর IP address (localhost)
- `:3000` = Port number (একটা door number মত)

**সহজ ভাষায়:**
```
আপনার Computer
├── Port 80 (সাধারণত web browser)
├── Port 3000 ← Backend এখানে চলছে
├── Port 5432 (PostgreSQL থাকলে)
└── আরো হাজারো ports...
```

#### 2. Browser এ চলবে কি?

**হ্যাঁ, চলবে!** কিন্তু শুধু test করার জন্য।

**Test করুন:**
1. Backend চালু রাখুন
2. Browser খুলুন (Chrome/Firefox)
3. Address bar এ লিখুন: `http://127.0.0.1:3000/health`
4. Enter press করুন

**দেখবেন:**
```
OK
```

এর মানে backend কাজ করছে!

#### 3. এটা কি API link?

**হ্যাঁ!** এটা API server এর base address।

**Full API links:**
```
http://127.0.0.1:3000/api/cards              ← Cards list
http://127.0.0.1:3000/api/cards/1/lines      ← Card 1 এর lines
http://127.0.0.1:3000/api/cards/1/meta       ← Card 1 এর info
http://127.0.0.1:3000/health                 ← Health check
```

#### 4. API কে create করলো?

**Axum!** (Rust এর web framework)

**Code এ দেখুন:** `backend/src/main.rs`

```rust
// Axum server তৈরি করছে
let app = Router::new()
    .route("/health", get(health_check))           // Health endpoint
    .route("/api/cards", get(get_cards))           // Cards list
    .route("/api/cards/:id/lines", get(get_lines)) // Lines fetch
    // ... আরো routes
```

**Axum এর কাজ:**
1. Port 3000 এ listen করা (wait করা requests এর জন্য)
2. Request আসলে সঠিক function call করা
3. Response পাঠানো

#### 5. API দিয়ে কি পাঠায়?

**Request → Response flow:**

**Example 1: Card list চাওয়া**
```
Request:
GET http://127.0.0.1:3000/api/cards

Response:
{
  "cards": [
    {
      "id": 1,                    ← User/Card ID
      "title": "My Notes",
      "total_lines": 1000
    }
  ]
}
```

**Example 2: Specific lines চাওয়া**
```
Request:
GET http://127.0.0.1:3000/api/cards/1/lines?start_line=100&limit=5

Parameters পাঠাচ্ছে:
- card_id = 1        ← কোন card?
- start_line = 100   ← কোন line থেকে?
- limit = 5          ← কতগুলো line?

Response:
[
  {"line_number": 100, "line_text": "Line 100 content"},
  {"line_number": 101, "line_text": "Line 101 content"},
  {"line_number": 102, "line_text": "Line 102 content"},
  {"line_number": 103, "line_text": "Line 103 content"},
  {"line_number": 104, "line_text": "Line 104 content"}
]
```

#### 6. Database এ ID ধরে data খোঁজে কিভাবে?

**Complete Flow:**

```
┌─────────────────────────────────────────────────────────────┐
│ Step 1: Frontend Request পাঠায়                              │
│ GET /api/cards/1/lines?start_line=100&limit=5               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 2: Axum Backend Request receive করে                    │
│ - card_id = 1 extract করে                                  │
│ - start_line = 100 extract করে                             │
│ - limit = 5 extract করে                                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 3: Backend SQL Query তৈরি করে                         │
│                                                             │
│ SELECT line_number, line_text                               │
│ FROM card_chunks                                            │
│ WHERE card_id = 1                    ← ID দিয়ে filter      │
│   AND line_number >= 100             ← Start line          │
│ ORDER BY line_number                                        │
│ LIMIT 5                              ← কতগুলো চাই         │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 4: SQLite Database query execute করে                  │
│                                                             │
│ Database file: backend/data/app.db                          │
│                                                             │
│ Table: card_chunks                                          │
│ ┌────────┬─────────────┬──────────────────┐                │
│ │card_id │ line_number │ line_text        │                │
│ ├────────┼─────────────┼──────────────────┤                │
│ │   1    │     100     │ "Line 100..."    │ ← Match!       │
│ │   1    │     101     │ "Line 101..."    │ ← Match!       │
│ │   1    │     102     │ "Line 102..."    │ ← Match!       │
│ │   1    │     103     │ "Line 103..."    │ ← Match!       │
│ │   1    │     104     │ "Line 104..."    │ ← Match!       │
│ │   1    │     105     │ "Line 105..."    │ (Limit 5 so stop)│
│ │   2    │     100     │ "Other card..."  │ (card_id ≠ 1)  │
│ └────────┴─────────────┴──────────────────┘                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 5: Backend JSON response তৈরি করে                     │
│                                                             │
│ [                                                           │
│   {"line_number": 100, "line_text": "Line 100..."},        │
│   {"line_number": 101, "line_text": "Line 101..."},        │
│   {"line_number": 102, "line_text": "Line 102..."},        │
│   {"line_number": 103, "line_text": "Line 103..."},        │
│   {"line_number": 104, "line_text": "Line 104..."}         │
│ ]                                                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 6: Frontend JSON parse করে screen এ দেখায়            │
│                                                             │
│ ┌─────────────────────────────────────┐                    │
│ │ 100 │ Line 100...                   │                    │
│ │ 101 │ Line 101...                   │                    │
│ │ 102 │ Line 102...                   │                    │
│ │ 103 │ Line 103...                   │                    │
│ │ 104 │ Line 104...                   │                    │
│ └─────────────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
```

#### 7. Code এ দেখুন:

**Backend code:** `backend/src/routes/lines.rs`

```rust
pub async fn get_card_lines(
    Path(card_id): Path<String>,           // ← URL থেকে card_id নেয়
    Query(params): Query<FetchLinesQuery>, // ← Query params নেয়
) -> Result<Json<Vec<LineEntry>>> {
    
    // SQL query execute করে
    let lines = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT line_number, line_text
        FROM card_chunks
        WHERE card_id = $1              -- ← card_id দিয়ে filter
          AND line_number >= $2         -- ← start_line
        ORDER BY line_number ASC
        LIMIT $3                        -- ← limit
        "#,
    )
    .bind(&card_id)                     // ← $1 = card_id
    .bind(params.start_line)            // ← $2 = start_line
    .bind(params.limit)                 // ← $3 = limit
    .fetch_all(&state.db_pool)          // ← Database থেকে fetch
    .await?;
    
    // JSON response return করে
    Ok(Json(lines))
}
```

#### 8. Summary:

| Component | কি করে? |
|-----------|---------|
| `127.0.0.1:3000` | Backend এর address (আপনার computer এ) |
| Axum | API server তৈরি করে, requests handle করে |
| API | Frontend ↔ Backend communication |
| card_id | Database এ specific card খুঁজতে ব্যবহার হয় |
| SQL Query | Database থেকে data fetch করে |
| JSON | Data পাঠানোর format |

**মনে রাখুন:**
- সব আপনার computer এ হচ্ছে
- Internet লাগছে না
- Browser শুধু test করার জন্য
- আসল app egui (desktop app)

</details>

---

#### Step 2: Card তৈরি করুন

নতুন terminal খুলুন (Terminal 2):

```bash
# Windows PowerShell
curl -X POST http://localhost:3000/api/cards -H "Content-Type: application/json" -d '{\"title\": \"Test Card\"}'
```

**Response দেখবেন:**
```json
{
  "id": 1,
  "title": "Test Card",
  "total_lines": 0
}
```

এর মানে card তৈরি হয়েছে, ID = 1।

---

#### Step 3: Test Data তৈরি করুন

```bash
# 1000 lines এর একটা file তৈরি করুন
1..1000 | ForEach-Object { "Line number $_" } | Out-File test.txt -Encoding UTF8
```

এটা `test.txt` file তৈরি করবে 1000 lines সহ।

---

#### Step 4: CLI দিয়ে Data Insert করুন

```bash
# Terminal 2 এ
cd cli
cargo run -- --card-id 1 --file ../test.txt
```

**দেখবেন:**
```
📥 Starting data ingestion...
━━━━━━━━━━━━━━━━━━━━━━ 1000/1000 lines
✅ Complete! 1000 lines inserted
```

এখন data database এ save হয়ে গেছে।

---

#### Step 5: API দিয়ে Check করুন

```bash
# Check করুন data আছে কিনা
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

এর মানে 1000 lines successfully insert হয়েছে!

এখন কিছু lines দেখুন:

```bash
curl "http://localhost:3000/api/cards/1/lines?start_line=0&limit=5"
```

**Response:**
```json
[
  {"line_number": 0, "line_text": "Line number 1"},
  {"line_number": 1, "line_text": "Line number 2"},
  {"line_number": 2, "line_text": "Line number 3"},
  {"line_number": 3, "line_text": "Line number 4"},
  {"line_number": 4, "line_text": "Line number 5"}
]
```

Perfect! Data আছে।

---

#### Step 6: Frontend Start করুন

```bash
# Terminal 3 খুলুন
cargo run
```

Frontend window খুলবে।

**Frontend এ:**
1. "Open Virtual Scroller" button click করুন
2. Card ID field এ `1` লিখুন
3. "Load Card" button click করুন
4. Scroll করুন!

---

## ❓ Question 5: API মানে কি?

**Your Question:**
> API সম্পর্কে আমার ধারণা নেই।

**Answer:**

### API = Application Programming Interface

**সহজ ভাষায়:** একটা program অন্য program এর সাথে কথা বলার নিয়ম।

### Real Life Example:

আপনি restaurant এ গেলেন:

```
You (Customer) → Waiter → Kitchen → Waiter → You (Food)
```

এখানে **Waiter = API**

- আপনি directly kitchen এ যান না
- Waiter এর মাধ্যমে order দেন
- Waiter kitchen থেকে খাবার নিয়ে আসে

### এই Project এ:

```
Frontend → Backend API → Database → Backend API → Frontend
```

**Frontend = Customer**
**Backend API = Waiter**
**Database = Kitchen**

### API Endpoints (Menu):

| Endpoint | কি করে? |
|----------|---------|
| GET /api/cards | সব cards এর list |
| GET /api/cards/1/lines | Card 1 এর lines |
| POST /api/cards | নতুন card তৈরি |
| PUT /api/cards/1/lines/5 | Line 5 edit করা |

### Example Request:

```bash
curl http://localhost:3000/api/cards/1/lines?start_line=100&limit=10
```

**Breaking it down:**
- `http://localhost:3000` = Backend এর address
- `/api/cards/1/lines` = Endpoint (কোন function call করবেন)
- `?start_line=100&limit=10` = Parameters (কি চাচ্ছেন)

**Response:**
```json
[
  {"line_number": 100, "line_text": "..."},
  {"line_number": 101, "line_text": "..."}
]
```

---

## ❓ Question 6: Internet লাগবে কি?

**Your Question:**
> HTTP request মানে কি internet লাগবে?

**Answer:**

**না! Internet লাগবে না।**

### Localhost = Your Computer

```
localhost = 127.0.0.1 = Your own computer
```

### Test করুন:

1. **Internet ON করে test করুন:**
   - Backend চালু করুন
   - Frontend চালু করুন
   - কাজ করবে ✅

2. **Internet OFF করে test করুন:**
   - WiFi disconnect করুন
   - Backend চালু করুন
   - Frontend চালু করুন
   - এখনও কাজ করবে ✅

কারণ সব communication আপনার computer এর ভিতরেই হচ্ছে।

### Network Path:

```
Frontend (Process 1)
    ↓
Operating System (Windows)
    ↓
Loopback Interface (127.0.0.1)
    ↓
Operating System (Windows)
    ↓
Backend (Process 2)
```

এটা সব আপনার computer এর RAM এ হয়, network card ব্যবহার হয় না!

---

## ❓ Question 7: Database file কোথায় দেখব?

**Your Question:**
> app.db file টা কোথায়? কিভাবে দেখব?

**Answer:**

### Location:

```
Your Project/
└── backend/
    └── data/
        └── app.db  ← এখানে
```

### File Explorer দিয়ে দেখুন:

1. Project folder খুলুন
2. `backend` folder এ যান
3. `data` folder এ যান
4. `app.db` file দেখবেন

### File Size:

- Empty database: ~20 KB
- 1000 lines: ~100 KB
- 1 million lines: ~100 MB
- 1 billion lines: ~100 GB

### View করতে চান?

**Download করুন:** [DB Browser for SQLite](https://sqlitebrowser.org/)

**Steps:**
1. DB Browser install করুন
2. Open Database → `backend/data/app.db` select করুন
3. Browse Data tab এ যান
4. Table select করুন: `cards` বা `card_chunks`
5. Data দেখতে পাবেন!

---

## ❓ Question 8: Memory কোথায় ব্যবহার হচ্ছে?

**Your Question:**
> 500 MB RAM constraint আছে, কোথায় কত memory ব্যবহার হচ্ছে?

**Answer:**

### Memory Breakdown:

```
Operating System:        ~200 MB
Backend (Axum):          ~15 MB
Frontend (egui):         ~25 MB
SQLite Cache:            ~10 MB
Other processes:         ~250 MB
─────────────────────────────────
Total:                   ~500 MB ✅
```

### Check করুন:

**Windows Task Manager:**
1. `Ctrl + Shift + Esc` press করুন
2. Processes tab এ যান
3. Find করুন:
   - `backend.exe` → ~15 MB
   - `frontend.exe` → ~25 MB

### কেন এত কম?

**Virtual Scrolling!**

```
Traditional approach:
Load 1 billion lines → 100 GB RAM ❌

Our approach:
Load only 50 visible lines → 10 KB RAM ✅
```

---

## 📝 Summary

### Key Points:

1. **HTTP ≠ Internet**
   - HTTP localhost এ কাজ করে
   - Internet লাগে না

2. **Data Storage**
   - Location: `backend/data/app.db`
   - Type: SQLite (local file)
   - NOT online, NOT PostgreSQL

3. **API**
   - Frontend ↔ Backend communication
   - JSON format
   - Localhost only

4. **Memory**
   - Total: < 60 MB
   - Virtual scrolling = efficient
   - Only visible data in RAM

5. **Testing**
   - Backend → CLI → Frontend
   - All offline
   - No internet needed

---

## 🎯 Next Questions?

এই file এ আপনার প্রশ্ন এবং উত্তর add হবে।

Format:
```
## ❓ Question X: [Your Question]

**Your Question:**
> [Question details]

**Answer:**
[Detailed answer]
```

---

## ❓ Question 9: Database empty কেন? Virtual scrolling কাজ করছে না!

**Your Question:**
> আমি database দেখেছি `C:\Users\USER\OneDrive\Desktop\Irak_Off_Days\Note_Task\backend\data\app.db` - এটা blank! Virtual scrolling এর জন্য তো data লাগবে, তাহলে কিভাবে 100 line display হবে?

**Answer:**

### Database Empty কারণ:

**এখনো কোনো data insert করা হয়নি!**

Virtual scrolling system তৈরি হয়ে গেছে, কিন্তু এটা কাজ করবে যখন:

```
Step 1: Backend চালু করুন
Step 2: Card তৈরি করুন (API দিয়ে)
Step 3: CLI দিয়ে data insert করুন
Step 4: Frontend থেকে virtual scroller open করুন
Step 5: তখন 100 lines display হবে!
```

### এখন কি করতে হবে:

#### 1. Backend Start করুন:

```bash
cd backend
cargo run
```

**Wait করুন যতক্ষণ না দেখেন:**
```
🌐 Server listening on http://127.0.0.1:3000
```

---

#### 2. Card তৈরি করুন:

নতুন terminal খুলুন:

```bash
# Windows PowerShell
curl -X POST http://localhost:3000/api/cards `
  -H "Content-Type: application/json" `
  -d '{\"title\": \"My First Card\"}'
```

**Response দেখবেন:**
```json
{
  "id": "abc-123-xyz",
  "title": "My First Card",
  "total_lines": 0
}
```

**এই `id` টা copy করুন!** (যেমন: `abc-123-xyz`)

---

#### 3. Test Data তৈরি করুন:

```bash
# 1000 lines এর একটা file তৈরি করুন
1..1000 | ForEach-Object { "This is line number $_" } | Out-File test_data.txt -Encoding UTF8
```

---

#### 4. CLI দিয়ে Data Insert করুন:

```bash
cd cli
cargo run -- --card-id "abc-123-xyz" --file ../test_data.txt
```

**Replace করুন:** `abc-123-xyz` আপনার actual card ID দিয়ে!

**দেখবেন:**
```
📥 Starting data ingestion...
━━━━━━━━━━━━━━━━━━━━━━ 1000/1000 lines
✅ Complete! 1000 lines inserted
```

---

#### 5. Database Check করুন:

এখন database empty না!

**Check করুন:**
```bash
curl http://localhost:3000/api/cards/abc-123-xyz/meta
```

**Response:**
```json
{
  "card_id": "abc-123-xyz",
  "title": "My First Card",
  "total_lines": 1000  ← এখন data আছে!
}
```

---

#### 6. Frontend এ দেখুন:

```bash
# নতুন terminal
cargo run
```

Frontend window এ:
1. "Open Virtual Scroller" button click করুন
2. Card ID field এ `abc-123-xyz` paste করুন
3. "Load Card" button click করুন
4. **এখন scroll করুন - 100 lines display হবে!**

---

### Virtual Scrolling কিভাবে কাজ করে:

```
Database এ আছে: 1000 lines (সব data)
         ↓
Frontend শুধু চায়: Line 0-100 (visible lines)
         ↓
Backend পাঠায়: শুধু 100 lines (JSON)
         ↓
Frontend display করে: 100 lines
         ↓
User scroll করলে: নতুন 100 lines fetch করে
```

**Memory Usage:**
- Database: 1000 lines stored (disk এ)
- RAM: শুধু 100 lines (10 KB)

---

## ❓ Question 10: "Text too large, truncating to 2048 pixels" error এর মানে কি?

**Your Question:**
> এই error দেখাচ্ছে:
> ```
> ⚠️ Text too large (132335.81 pixels), truncating to 2048 pixels
> ⚠️ Text too large (135125.34 pixels), truncating to 2048 pixels
> ```
> এর মানে কি? 2048 pixel কেন? Virtual scrolling কাজ করছে না?

**Answer:**

### এটা Virtual Scrolling এর Error না!

এটা আপনার **existing app** এর quote/text rendering এর warning।

### কি হচ্ছে:

```
Your App এ একটা quote/text আছে যেটা অনেক বড়
         ↓
egui library সেটা render করতে গিয়ে দেখছে:
"এই text render করলে 132335 pixels লাগবে!"
         ↓
egui বলছে: "এত বড় text render করা যাবে না,
আমি শুধু 2048 pixels পর্যন্ত দেখাবো"
         ↓
Text truncate (কেটে ফেলা) হচ্ছে
```

### 2048 Pixels মানে কি?

**Pixels = Screen এ কত জায়গা নিবে**

Example:
- 1 line text = ~20 pixels height
- 100 lines = ~2000 pixels
- 2048 pixels = ~100 lines display করা যায়

### 132335 Pixels মানে:

```
132335 pixels ÷ 20 pixels per line = ~6600 lines!
```

আপনার app একসাথে 6600 lines render করার চেষ্টা করছে - এটা করা যাবে না!

### এটা কোথায় হচ্ছে?

**Your existing app এর quote display section এ।**

এটা virtual scrolling এর সাথে related না। এটা আলাদা feature।

### Solution:

#### Option 1: Quote Text Limit করুন

আপনার existing code এ যেখানে quote display হয়:

```rust
// Before (সব text একসাথে)
ui.label(&quote_text);

// After (শুধু first 1000 characters)
let display_text = if quote_text.len() > 1000 {
    format!("{}...", &quote_text[..1000])
} else {
    quote_text.clone()
};
ui.label(&display_text);
```

#### Option 2: ScrollArea ব্যবহার করুন

```rust
egui::ScrollArea::vertical()
    .max_height(400.0)  // Maximum 400 pixels height
    .show(ui, |ui| {
        ui.label(&quote_text);
    });
```

#### Option 3: Virtual Scrolling ব্যবহার করুন!

যদি quote text অনেক বড় হয়, তাহলে সেটাও virtual scrolling দিয়ে display করুন:

```rust
// Quote কে card হিসেবে save করুন
// Virtual scroller দিয়ে display করুন
```

---

### Summary:

| Issue | What it is | Solution |
|-------|-----------|----------|
| Database empty | No data inserted yet | Use CLI to insert data |
| Text truncation | Existing app trying to render huge text | Add ScrollArea or limit text |
| Virtual scrolling | New feature, works separately | Follow testing steps |

---

### Quick Fix for Text Truncation:

আপনার main.rs এ যেখানে quote display হয়, সেখানে খুঁজুন:

```rust
ui.label(&quote_text);
```

Replace করুন:

```rust
egui::ScrollArea::vertical()
    .max_height(400.0)
    .show(ui, |ui| {
        ui.label(&quote_text);
    });
```

এটা warning fix করবে এবং বড় text scroll করা যাবে।

---

## 📝 Summary

### Virtual Scrolling Setup:

```
1. Backend চালু করুন ✅
2. Card তৈরি করুন ✅
3. CLI দিয়ে data insert করুন ✅
4. Frontend এ virtual scroller open করুন ✅
5. Scroll করুন - শুধু 100 lines display হবে! ✅
```

### Text Truncation Fix:

```
1. main.rs এ quote display section খুঁজুন
2. ScrollArea add করুন
3. Warning চলে যাবে ✅
```

---

## ❓ Question 11: Frontend থেকে Database এ save - Direct নাকি API দিয়ে?

**Your Question:**
> Frontend থেকে data কি direct database এ save হবে? নাকি Axum API দিয়ে save হবে? যদি API দিয়ে হয়, তাহলে Axum এর কাজ কি?

**Answer:**

### ✅ API দিয়ে Save হয় (Direct না!)

**Frontend কখনো direct database access করে না।**

### Complete Flow:

```
┌─────────────────────────────────────────────────────────────┐
│                    Data Save Flow                            │
└─────────────────────────────────────────────────────────────┘

Step 1: User types text in Frontend
┌──────────────────┐
│   Frontend       │
│   (egui app)     │
│                  │
│  User লিখছে:     │
│  "Line 1         │
│   Line 2         │
│   Line 3"        │
└────────┬─────────┘
         │
         │ User saves (Enter press)
         ▼
Step 2: Frontend prepares HTTP request
┌──────────────────┐
│   Frontend       │
│                  │
│  Data তৈরি করে:  │
│  {               │
│    "lines": [    │
│      {           │
│        "line_number": 0,
│        "line_text": "Line 1"
│      },          │
│      ...         │
│    ]             │
│  }               │
└────────┬─────────┘
         │
         │ HTTP POST request
         │ URL: http://localhost:3000/api/cards/quote_0/lines/batch
         │ Body: JSON data
         ▼
Step 3: Axum Backend receives request
┌──────────────────┐
│   Axum Backend   │
│   (Port 3000)    │
│                  │
│  Request আসলো!   │
│  Route match:    │
│  POST /api/cards/:id/lines/batch
│                  │
│  Function call:  │
│  batch_insert_lines()
└────────┬─────────┘
         │
         │ Parse JSON
         │ Validate data
         ▼
Step 4: Backend queries Database
┌──────────────────┐
│   Axum Backend   │
│                  │
│  SQL query তৈরি:  │
│  INSERT INTO card_chunks
│  (card_id, line_number, line_text)
│  VALUES (?, ?, ?)
│                  │
│  Execute query   │
└────────┬─────────┘
         │
         │ SQL query
         ▼
Step 5: SQLite saves data
┌──────────────────┐
│   SQLite DB      │
│   (app.db file)  │
│                  │
│  Data save হলো:  │
│  ┌─────┬────┬────┐
│  │card │line│text│
│  ├─────┼────┼────┤
│  │quo_0│ 0  │Lin1│
│  │quo_0│ 1  │Lin2│
│  │quo_0│ 2  │Lin3│
│  └─────┴────┴────┘
└────────┬─────────┘
         │
         │ Success response
         ▼
Step 6: Backend sends response to Frontend
┌──────────────────┐
│   Axum Backend   │
│                  │
│  Response:       │
│  Status: 201     │
│  Body: {         │
│    "success": true
│  }               │
└────────┬─────────┘
         │
         │ HTTP response
         ▼
Step 7: Frontend receives confirmation
┌──────────────────┐
│   Frontend       │
│                  │
│  Console log:    │
│  "✅ Saved 3 lines"
└──────────────────┘
```

---

### কেন Direct Database Access করে না?

#### ❌ Direct Access (যদি করতাম):

```
Frontend → SQLite file (app.db)
```

**Problems:**
1. **Security:** Frontend থেকে কেউ যেকোনো data delete/modify করতে পারবে
2. **File Lock:** SQLite file একসাথে multiple process access করতে পারে না
3. **Validation:** কোনো data validation নেই
4. **Cross-platform:** Browser থেকে local file access করা যায় না
5. **Network:** Remote database access করা যাবে না

#### ✅ API Access (আমরা যা করছি):

```
Frontend → Axum API → SQLite
```

**Benefits:**
1. **Security:** Axum validate করে, unauthorized access block করে
2. **Centralized:** একটা মাত্র process (Axum) database access করে
3. **Validation:** Data check করে save করার আগে
4. **Scalable:** পরে PostgreSQL/MySQL এ switch করা সহজ
5. **Network:** Remote server এও কাজ করবে

---

### Axum এর কাজ কি?

**Axum = Middleman (মধ্যস্থতাকারী)**

```
Frontend ←→ Axum ←→ Database
```

#### Axum এর Responsibilities:

##### 1. Request Handling
```rust
// Frontend থেকে request receive করে
POST /api/cards/quote_0/lines/batch
Body: {"lines": [...]}
```

##### 2. Data Validation
```rust
// Check করে data ঠিক আছে কিনা
if lines.is_empty() {
    return Error("Lines cannot be empty");
}
if lines.len() > 10000 {
    return Error("Too many lines");
}
```

##### 3. Database Operations
```rust
// SQL query execute করে
INSERT INTO card_chunks (card_id, line_number, line_text)
VALUES ($1, $2, $3)
```

##### 4. Error Handling
```rust
// Database error handle করে
match db.execute(query) {
    Ok(_) => return Success,
    Err(e) => return Error(e)
}
```

##### 5. Response Formatting
```rust
// Frontend কে response পাঠায়
{
    "success": true,
    "lines_inserted": 3
}
```

---

### Code Example:

#### Frontend Code (main.rs):
```rust
fn save_to_backend_database(&self, quote_idx: usize, text: &str) {
    let url = "http://localhost:3000/api/cards/quote_0/lines/batch";
    
    // HTTP request পাঠায় (API call)
    let client = reqwest::blocking::Client::new();
    client.post(url)
        .json(&data)
        .send()  // ← এখানে Axum এ যাচ্ছে
}
```

#### Backend Code (backend/src/routes/lines.rs):
```rust
pub async fn batch_insert_lines(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BatchInsertRequest>,
) -> AppResult<StatusCode> {
    // 1. Validate
    if payload.lines.is_empty() {
        return Err(AppError::InvalidInput("Empty lines"));
    }
    
    // 2. Database query
    for line in &payload.lines {
        sqlx::query(
            "INSERT INTO card_chunks (card_id, line_number, line_text) 
             VALUES ($1, $2, $3)"
        )
        .bind(&card_id)
        .bind(line.line_number)
        .bind(&line.line_text)
        .execute(&state.db_pool)  // ← এখানে Database এ যাচ্ছে
        .await?;
    }
    
    // 3. Response
    Ok(StatusCode::CREATED)
}
```

---

### Real-World Analogy:

#### Direct Database Access (❌):
```
You → Bank Vault (direct access)
```
- আপনি সরাসরি vault এ ঢুকে টাকা নিচ্ছেন
- কোনো security নেই
- কোনো record নেই
- Chaos!

#### API Access (✅):
```
You → Bank Teller → Bank Vault
```
- আপনি teller কে বলছেন
- Teller verify করছে (ID check)
- Teller vault থেকে টাকা নিচ্ছে
- Teller record রাখছে
- Safe and organized!

**এখানে:**
- You = Frontend
- Bank Teller = Axum API
- Bank Vault = Database

---

### Summary Table:

| Aspect | Direct Access | API Access (Axum) |
|--------|--------------|-------------------|
| Security | ❌ None | ✅ Validated |
| Validation | ❌ None | ✅ Yes |
| Error Handling | ❌ Manual | ✅ Automatic |
| Scalability | ❌ Limited | ✅ Easy |
| Multi-user | ❌ Conflicts | ✅ Handled |
| Network | ❌ Local only | ✅ Remote possible |

---

### আপনার Code এ কি হচ্ছে:

```rust
// main.rs (Frontend)
fn save_to_backend_database(...) {
    let url = format!("{}/api/cards/{}/lines/batch", backend_url, card_id);
    
    client.post(&url)  // ← API call (Axum এ যাচ্ছে)
        .json(&batch_data)
        .send()
}
```

**এটা:**
1. ❌ Direct database access করছে না
2. ✅ Axum API call করছে
3. ✅ Axum database এ save করছে

---

### Axum ছাড়া কি সম্ভব?

**হ্যাঁ, কিন্তু:**

```rust
// Frontend থেকে direct SQLite access
use rusqlite::Connection;

let conn = Connection::open("backend/data/app.db")?;
conn.execute("INSERT INTO ...", params![])?;
```

**Problems:**
1. Frontend এবং Backend দুইটাই একই file access করবে → File lock error
2. No validation
3. No security
4. No error handling
5. Can't work remotely

**তাই Axum API ব্যবহার করা best practice!**

---

**Last Updated:** 2026-03-26
