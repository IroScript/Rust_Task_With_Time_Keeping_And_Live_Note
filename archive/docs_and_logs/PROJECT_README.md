# Motivational Quotes App with Cloud Sync

A pure Rust desktop application for displaying motivational quotes with cloud synchronization.

## Quick Start

### Prerequisites
- Rust (latest stable)
- PostgreSQL database (or use Aiven free tier)

### 1. Clone the Repository
```bash
git clone <your-repo-url>
cd RustTask
```

### 2. Setup Kiro Agent (Optional)
If you use Kiro, run the setup script to add trusted commands:
```powershell
.\setup-kiro.ps1
```

### 3. Setup Backend

```bash
cd backend
cp .env.example .env
# Edit .env with your database credentials
cargo run --release
```

Backend will start on `http://localhost:3000`

### 2. Run Desktop App

```bash
cargo run --release
```

### 3. Create Profile

1. Click the profile icon (👤) in the title bar
2. Fill in your information
3. Click "Save & Connect"
4. Copy your User ID from the console
5. Open profile again and paste your User ID
6. Click "Save & Connect" again

### 4. Add Quotes

Your quotes will automatically sync to the cloud!

## Documentation

- [How It Works](HOW_IT_WORKS.md) - Architecture and flow diagrams
- [Quote Sync Guide](QUOTE_SYNC_GUIDE.md) - User guide for syncing
- [Integration Guide](INTEGRATION_GUIDE.md) - Integration overview
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md) - Technical details
- [Backend API](backend/API_TESTING.md) - API documentation

## Project Structure

```
.
├── src/
│   └── main.rs              # Desktop app (frontend)
├── backend/
│   ├── src/
│   │   ├── main.rs          # API server entry point
│   │   ├── routes/          # API endpoints
│   │   ├── models/          # Data models
│   │   └── ...
│   ├── migrations/          # Database schema
│   └── .env                 # Database credentials (not in git)
├── assets/                  # Fonts and resources
├── .kiro/specs/            # Feature specifications
└── docs/                    # Documentation
```

## Current Status

### ✅ Implemented
- User profile creation and storage
- Quote sync to backend
- PostgreSQL cloud storage
- REST API for users and documents
- Profile icon and modal UI
- Local + cloud persistence

### 🔲 Coming Soon
- Load quotes from backend on startup
- Real-time WebSocket sync
- Offline mode with SQLite
- CRDT conflict resolution
- Multi-device sync

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Check Your Data
```powershell
.\CHECK_YOUR_DATA.ps1
```

## Database Schema

### users
- id (UUID, primary key)
- name (TEXT)
- email (TEXT, unique)
- country_code (TEXT)
- company_name (TEXT)
- created_at (TIMESTAMPTZ)
- updated_at (TIMESTAMPTZ)

### documents
- id (UUID, primary key)
- user_id (UUID, foreign key)
- title (TEXT)
- content (BYTEA) - CRDT state
- crdt_version (BIGINT)
- created_at (TIMESTAMPTZ)
- updated_at (TIMESTAMPTZ)

## API Endpoints

### Users
- `POST /api/users` - Create user
- `GET /api/users/:id` - Get user
- `PUT /api/users/:id` - Update user

### Documents/Quotes
- `POST /api/documents` - Create quote
- `GET /api/documents` - List quotes
- `GET /api/documents/:id` - Get quote
- `PUT /api/documents/:id` - Update quote
- `DELETE /api/documents/:id` - Delete quote

### Health
- `GET /health` - Health check

## Contributing

This is a personal project, but suggestions are welcome!

## License

MIT License

## Credits

Built with ❤️ using pure Rust
