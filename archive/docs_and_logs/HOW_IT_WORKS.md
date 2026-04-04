# How Your App Works - Simple Explanation

## The Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  YOUR DESKTOP APP (Motivational Quotes)                         │
│  - Shows quotes on your screen                                  │
│  - Has a profile icon (👤) in title bar                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ When you click profile icon
                              │ and fill in your info
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  PROFILE MODAL                                                   │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Name:     Irak                                            │ │
│  │ Email:    md.kamruzzamanirak@gmail.com                    │ │
│  │ Country:  BD                                              │ │
│  │ Company:  My Company                                      │ │
│  │ Backend:  http://localhost:3000                           │ │
│  │                                                           │ │
│  │           [Save & Connect]  [Cancel]                      │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Sends HTTP POST request
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  RUST BACKEND (Running on localhost:3000)                       │
│  - Receives your profile data                                   │
│  - Creates user account                                         │
│  - Saves to database                                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Stores in cloud
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  POSTGRESQL DATABASE (Aiven Cloud)                              │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ users table:                                              │ │
│  │ ┌────────────────────────────────────────────────────┐   │ │
│  │ │ id: 1cc88e68-896e-4fdb-abee-c27481343d83          │   │ │
│  │ │ name: Irak                                         │   │ │
│  │ │ email: md.kamruzzamanirak@gmail.com               │   │ │
│  │ │ country_code: BD                                   │   │ │
│  │ │ company_name: My Company                           │   │ │
│  │ │ created_at: 2026-02-27 18:27:14                   │   │ │
│  │ └────────────────────────────────────────────────────┘   │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## What Happened When You Created Your Profile

### Step 1: You Filled the Form
- Name: Irak
- Email: md.kamruzzamanirak@gmail.com
- Country: BD
- Company: My Company

### Step 2: Desktop App Sent Request
Your app sent this JSON to the backend:
```json
{
  "name": "Irak",
  "email": "md.kamruzzamanirak@gmail.com",
  "country_code": "BD",
  "company_name": "My Company"
}
```

### Step 3: Backend Processed It
The backend:
1. Received the request at `POST /api/users`
2. Validated the data
3. Generated a unique ID: `1cc88e68-896e-4fdb-abee-c27481343d83`
4. Saved to PostgreSQL database

### Step 4: Database Stored It
Your data is now in the cloud database on Aiven servers!

## What Each Part Does

### 1. Desktop App (Frontend)
- **File**: `src/main.rs`
- **What it does**: Shows your motivational quotes
- **New feature**: Profile icon that opens a form
- **Technology**: Pure Rust with egui (GUI library)

### 2. Backend API (Server)
- **Location**: `backend/` folder
- **What it does**: 
  - Receives requests from your app
  - Validates data
  - Saves to database
  - Sends responses back
- **Running on**: `http://localhost:3000`
- **Technology**: Pure Rust with axum (web framework)

### 3. Database (Storage)
- **Service**: Aiven PostgreSQL (cloud)
- **What it does**: Stores your data permanently
- **Tables**:
  - `users` - Your profile information
  - `documents` - Your quotes/notes (future)
  - `presence` - Who's online (future)

## Why You Got an Error

When you tried to create your profile again, you saw:
```
duplicate key value violates unique constraint "users_email_key"
```

This means:
- ✅ Your email is already registered
- ✅ The database is protecting your data
- ✅ Each email can only be used once (security feature)

This is GOOD! It prevents duplicate accounts.

## What You Can Do Now

### 1. View Your User Data
Open PowerShell and run:
```powershell
Invoke-WebRequest -Uri http://localhost:3000/api/users/1cc88e68-896e-4fdb-abee-c27481343d83 -Method GET -UseBasicParsing | Select-Object -ExpandProperty Content
```

You'll see your profile data!

### 2. Create a Quote/Document
```powershell
$body = @{
    user_id = "1cc88e68-896e-4fdb-abee-c27481343d83"
    title = "My First Note"
    initial_content = "This is my first note!"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://localhost:3000/api/documents -Method POST -Body $body -ContentType "application/json" -UseBasicParsing
```

### 3. List Your Documents
```powershell
Invoke-WebRequest -Uri "http://localhost:3000/api/documents?user_id=1cc88e68-896e-4fdb-abee-c27481343d83" -Method GET -UseBasicParsing | Select-Object -ExpandProperty Content
```

## Next Steps - What We'll Build

Right now, your quotes in the desktop app are NOT synced to the backend yet. We need to:

1. ✅ User profile - DONE!
2. 🔲 Sync quotes when you add/edit them
3. 🔲 Load quotes from backend when app starts
4. 🔲 Real-time sync with WebSocket
5. 🔲 Offline mode with SQLite

## Files to Check

### Your Profile is Saved In:
1. **Local**: `settings.json` (in your app folder)
2. **Cloud**: PostgreSQL database on Aiven

### Backend Logs Show:
- User creation: `Created user: Irak (1cc88e68-896e-4fdb-abee-c27481343d83)`
- All API requests
- Database connections

## Summary

✅ Your desktop app is running
✅ Backend is running on port 3000
✅ Your profile is saved in the cloud database
✅ You can create documents/quotes via API
✅ Everything is pure Rust!

The connection is working perfectly! 🎉
