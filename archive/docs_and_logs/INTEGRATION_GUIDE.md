# Frontend-Backend Integration Guide

## What We Built

Your motivational quotes desktop app now has:
1. **Profile Icon** in the title bar (user icon)
2. **User Profile Modal** for entering your information
3. **Backend Connection** to sync data with the Rust backend

## How to Use

### Step 1: Start the Backend

Open a terminal and run:
```bash
cd backend
cargo run --release
```

The backend will start at `http://localhost:3000`

### Step 2: Run Your Desktop App

Open another terminal and run:
```bash
cargo run --release
```

### Step 3: Set Up Your Profile

1. Click the **user icon** (👤) in the title bar
2. Fill in your information:
   - Name: Your name
   - Email: Your email
   - Country: Country code (e.g., US, BD, IN)
   - Company: Your company name
   - Backend: `http://localhost:3000` (default)
3. Click **"Save & Connect"**

The app will:
- Save your profile locally
- Create your user account in the backend
- Connect to the PostgreSQL database (Aiven)

### Step 4: Your Quotes Are Now Synced!

Your quotes are stored in:
- **Local**: `settings.json` file
- **Cloud**: PostgreSQL database on Aiven

## What's Connected

✅ User profile saved locally and in cloud
✅ Backend API running on port 3000
✅ PostgreSQL (Aiven) storing user data
✅ Pure Rust stack (no JavaScript!)

## Next Steps (Future Features)

🔲 Sync quotes to backend when you add/edit them
🔲 WebSocket live sync (see quotes update in real-time)
🔲 Offline mode with SQLite cache
🔲 Multi-device sync

## Testing the Connection

You can verify the backend connection by checking the console output when you click "Save & Connect". You should see:
- ✅ User created successfully!

Or check the backend terminal to see the API request.

## Architecture

```
Desktop App (egui)
    ↓
User Profile Modal
    ↓
HTTP Request (reqwest)
    ↓
Backend API (axum)
    ↓
PostgreSQL (Aiven Cloud)
```

All pure Rust! 🦀
