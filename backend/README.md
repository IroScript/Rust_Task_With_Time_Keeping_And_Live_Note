# Pure Rust Backend - Live Editing with Offline Support

100% Pure Rust backend for real-time document editing with offline-first architecture.

## 🚀 Quick Start

```bash
# 1. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install sqlx-cli for database migrations
cargo install sqlx-cli --no-default-features --features postgres

# 3. Run database migrations
sqlx migrate run

# 4. Start the server
cargo run

# Server will start on http://localhost:3000
```

## 📋 Environment Variables

All configuration is in `.env` file (already configured with your Aiven PostgreSQL).

## 🔌 API Endpoints

### Health Check
- `GET /health` - Server health check

### Users (No Auth)
- `POST /api/users` - Create user
- `GET /api/users/:id` - Get user
- `PUT /api/users/:id` - Update user

### Documents
- `POST /api/documents` - Create document
- `GET /api/documents/:id` - Get document
- `PUT /api/documents/:id` - Update document
- `DELETE /api/documents/:id` - Delete document
- `GET /api/documents?user_id=uuid` - List user's documents

### WebSocket (Live Editing)
- `WS /ws/:document_id` - Real-time sync

## 🏗️ Project Structure

```
backend/
├── Cargo.toml           # Dependencies
├── .env                 # Configuration (Aiven PostgreSQL)
├── migrations/          # Database migrations
│   ├── 20240227000001_create_users_table.sql
│   ├── 20240227000002_create_documents_table.sql
│   └── 20240227000003_create_presence_table.sql
└── src/
    ├── main.rs          # Entry point
    ├── config.rs        # Environment config
    ├── db.rs            # PostgreSQL connection
    ├── error.rs         # Error handling
    ├── models/          # Data models
    │   ├── mod.rs
    │   ├── user.rs
    │   └── document.rs
    ├── routes/          # API endpoints
    │   ├── mod.rs
    │   ├── users.rs
    │   ├── documents.rs
    │   └── websocket.rs
    └── crdt/            # CRDT sync engine
        └── mod.rs
```

## ✅ Current Status

**Phase 1: Project Setup** ✅ COMPLETE
- [x] Cargo project initialized
- [x] Dependencies configured
- [x] Environment configuration
- [x] PostgreSQL connection setup
- [x] Database migrations created
- [x] Basic project structure

**Next: Phase 2 - Implement User & Document APIs**

## 🧪 Testing

```bash
# Run tests
cargo test

# Check compilation
cargo check

# Run with logging
RUST_LOG=debug cargo run
```

## 📦 Dependencies (100% Pure Rust)

- **tokio** - Async runtime
- **axum** - Web framework
- **sqlx** - PostgreSQL driver
- **rusqlite** - SQLite for offline cache
- **yrs** - CRDT library
- **serde** - Serialization
- **uuid** - UUID generation
- **chrono** - Date/time handling

All dependencies are pure Rust - no JavaScript, Python, or other languages!

## 🎯 Features

- ✅ Real-time WebSocket sync
- ✅ CRDT conflict resolution
- ✅ Offline-first with SQLite cache
- ✅ "Someone is typing..." presence
- ✅ Fast PostgreSQL storage (Aiven)
- ✅ No authentication (MVP)
- ✅ CORS enabled for frontend

## 📚 Next Steps

See `.kiro/specs/mvp-live-editing/tasks.md` for detailed implementation tasks.
