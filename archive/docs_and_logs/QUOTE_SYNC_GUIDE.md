# Quote Sync Guide - How to Sync Your Quotes

## Overview

Your motivational quotes app now syncs with the backend! Here's how it works:

## Setup (First Time)

### Step 1: Start Backend
```bash
cd backend
cargo run --release
```

### Step 2: Start Desktop App
```bash
cargo run --release
```

### Step 3: Create Your Profile

#### Option A: New User (First Time)
1. Click the **profile icon** (👤) in the title bar
2. Fill in your information:
   - Name: Your name
   - Email: Your email
   - Country: Country code (US, BD, IN, etc.)
   - Company: Your company name
   - Backend: `http://localhost:3000`
   - **User ID: Leave EMPTY**
3. Click **"Save & Connect"**
4. Check the console output - you'll see:
   ```
   ✅ User created! Your ID: 1cc88e68-896e-4fdb-abee-c27481343d83
   💡 Copy this ID and paste it in the User ID field to sync quotes!
   ```
5. **IMPORTANT**: Copy your User ID!
6. Click profile icon again and paste your User ID
7. Click "Save & Connect" again

#### Option B: Existing User
1. Click the **profile icon** (👤)
2. Fill in your information
3. **Paste your User ID** in the User ID field
4. Click **"Save & Connect"**
5. Your quotes will load from the backend!

## How Quote Sync Works

### When You Add a Quote

```
You type a new quote in the app
         ↓
App saves it locally (settings.json)
         ↓
App sends it to backend API
         ↓
Backend saves to PostgreSQL
         ↓
Quote is now in the cloud! ☁️
```

### What Gets Synced

- ✅ Main text (the quote)
- ✅ Sub text (supporting text)
- ✅ User ID (who created it)
- ✅ Timestamp (when created)

### Console Messages

When you add a quote, you'll see:
```
✅ Quote synced to backend!
```

Or if there's an issue:
```
⚠️ Failed to sync quote: 500 Internal Server Error
❌ Backend sync error: connection refused
```

## Testing Quote Sync

### 1. Add a Quote in Your App
- Open your desktop app
- Add a new quote using the control panel
- Check the console for sync confirmation

### 2. Verify in Backend
Open PowerShell and run:
```powershell
# Replace with YOUR user ID
$userId = "1cc88e68-896e-4fdb-abee-c27481343d83"

Invoke-WebRequest -Uri "http://localhost:3000/api/documents?user_id=$userId" -Method GET -UseBasicParsing | Select-Object -ExpandProperty Content
```

You should see your quotes!

### 3. Check Database
Your quotes are stored in PostgreSQL on Aiven cloud. The backend logs will show:
```
INFO backend::routes::documents: Created document: Your Quote Text (document-id)
```

## Current Features

### ✅ Implemented
1. User profile creation
2. User profile persistence (local + cloud)
3. Quote sync to backend when added
4. Backend API for quotes
5. PostgreSQL cloud storage

### 🔲 Coming Next
1. Load quotes from backend on app start
2. Edit quote sync
3. Delete quote sync
4. Real-time WebSocket sync
5. Offline mode with SQLite

## Troubleshooting

### "Backend sync error: connection refused"
- Make sure backend is running: `cd backend && cargo run --release`
- Check backend URL in profile: should be `http://localhost:3000`

### "Failed to sync quote: 404"
- Check your User ID is correct
- Verify user exists: `Invoke-WebRequest -Uri http://localhost:3000/api/users/YOUR-ID`

### "Failed to sync quote: 500"
- Check backend console for error details
- Usually means database connection issue

### No User ID After Creating Account
- Check the console output when you click "Save & Connect"
- The User ID is printed there
- Copy it and paste it in the profile modal

## File Locations

### Local Storage
- **Profile**: `settings.json` (in app folder)
- **Quotes**: `settings.json` (in app folder)

### Cloud Storage
- **Profile**: PostgreSQL on Aiven
- **Quotes**: PostgreSQL on Aiven (documents table)

## Example Workflow

1. **Morning**: Open app, see motivational quotes
2. **Add Quote**: Type new quote, it syncs to cloud
3. **Close App**: All data saved locally and in cloud
4. **Evening**: Open app on another device (future feature)
5. **See Same Quotes**: Loaded from cloud (future feature)

## API Endpoints Used

### Create User
```
POST http://localhost:3000/api/users
Body: { name, email, country_code, company_name }
```

### Create Quote/Document
```
POST http://localhost:3000/api/documents
Body: { user_id, title, initial_content }
```

### List Quotes
```
GET http://localhost:3000/api/documents?user_id=YOUR-ID
```

## Next Steps

Want to see your quotes in the cloud? Run:
```powershell
.\CHECK_YOUR_DATA.ps1
```

This script shows:
- Your user profile
- All your synced quotes
- Backend health status

## Summary

✅ Profile saved locally and in cloud
✅ New quotes automatically sync to backend
✅ Quotes stored in PostgreSQL (Aiven)
✅ Pure Rust stack (no JavaScript!)
🔲 Load quotes from backend (coming next)
🔲 Real-time sync (coming next)

Your quotes are now backed up in the cloud! 🎉
