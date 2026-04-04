# Backend Setup Guide
# ব্যাকএন্ড সেটআপ গাইড

## Quick Start (দ্রুত শুরু)

### 1. Environment Setup (পরিবেশ সেটআপ)

Backend folder এ `.env` file ইতিমধ্যে তৈরি আছে local SQLite database এর জন্য।

```bash
cd backend
```

`.env` file এ এই configuration আছে:
```env
DATABASE_URL=sqlite:data/task_note.db
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
CORS_ORIGIN=http://localhost:5173
RUST_LOG=info,backend=debug
```

### 2. Database Folder তৈরি

```bash
# Windows PowerShell
mkdir -Force data

# Or manually create 'data' folder inside backend directory
```

### 3. Run Backend (ব্যাকএন্ড চালান)

Development mode:
```bash
cargo run
```

Release mode (faster):
```bash
cargo run --release
```

### 4. Verify (যাচাই করুন)

Backend চললে আপনি দেখবেন:
```
🚀 Starting Pure Rust Backend with Axum API
📊 Database: SQLite (local)
✅ Server running on http://0.0.0.0:3000
```

---

## Cloud Database Setup (ক্লাউড ডাটাবেস সেটআপ)

### PostgreSQL (Aiven) ব্যবহার করতে চাইলে:

1. `.env` file এ SQLite line টি comment করুন:
```env
# DATABASE_URL=sqlite:data/task_note.db
```

2. PostgreSQL URL uncomment করে আপনার credentials দিন:
```env
DATABASE_URL=postgresql://avnadmin:your_password@your-host.aivencloud.com:10206/defaultdb?sslmode=require
```

3. Backend restart করুন

---

## Troubleshooting (সমস্যা সমাধান)

### Error: "DATABASE_URL must be set"
**সমাধান**: নিশ্চিত করুন `backend/.env` file আছে এবং DATABASE_URL সেট করা আছে।

### Error: "unable to open database file"
**সমাধান**: `backend/data` folder তৈরি করুন:
```bash
mkdir data
```

### Error: "address already in use"
**সমাধান**: Port 3000 ইতিমধ্যে ব্যবহৃত হচ্ছে। `.env` এ PORT পরিবর্তন করুন:
```env
SERVER_PORT=3001
```

---

## API Endpoints

Backend চললে এই endpoints available থাকবে:

- `GET /health` - Health check
- `POST /api/cards` - Create new card
- `GET /api/cards` - Get all cards
- `PUT /api/cards/:id` - Update card
- `DELETE /api/cards/:id` - Delete card

---

## Development Tips

### Database Reset করতে চাইলে:
```bash
# SQLite database file মুছে দিন
rm data/task_note.db

# Backend restart করলে নতুন database তৈরি হবে
cargo run
```

### Logs দেখতে চাইলে:
`.env` file এ RUST_LOG level পরিবর্তন করুন:
```env
RUST_LOG=debug,backend=trace  # More detailed logs
RUST_LOG=error                # Only errors
```

---

## GitHub থেকে Clone করার পর

1. Repository clone করুন
2. Backend folder এ যান: `cd backend`
3. `.env` file ইতিমধ্যে আছে (local SQLite configuration সহ)
4. Data folder তৈরি করুন: `mkdir data`
5. Backend চালান: `cargo run`

✅ সব কিছু automatically কাজ করবে!
