# API Testing Guide

## Server Status

The backend is running at: `http://localhost:3000`

## Health Check

```powershell
curl http://localhost:3000/health
```

## User API

### Create User (No Authentication Required)

```powershell
$body = @{
    name = "John Doe"
    email = "john@example.com"
    country_code = "US"
    company_name = "Test Company"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://localhost:3000/api/users `
    -Method POST `
    -Body $body `
    -ContentType "application/json" `
    -UseBasicParsing
```

Response (201 Created):
```json
{
  "id": "bb51bed5-28ed-4c59-b326-bfd628023c56",
  "name": "John Doe",
  "email": "john@example.com",
  "country_code": "US",
  "company_name": "Test Company",
  "created_at": "2026-02-27T18:06:41.235154Z",
  "updated_at": "2026-02-27T18:06:41.235154Z"
}
```

### Get User by ID

```powershell
Invoke-WebRequest -Uri http://localhost:3000/api/users/bb51bed5-28ed-4c59-b326-bfd628023c56 `
    -Method GET `
    -UseBasicParsing | Select-Object -ExpandProperty Content
```

### Update User

```powershell
$body = @{
    name = "Jane Doe"
    email = "jane@example.com"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://localhost:3000/api/users/bb51bed5-28ed-4c59-b326-bfd628023c56 `
    -Method PUT `
    -Body $body `
    -ContentType "application/json" `
    -UseBasicParsing
```

## Document API

### Create Document

```powershell
$body = @{
    user_id = "bb51bed5-28ed-4c59-b326-bfd628023c56"
    title = "My First Document"
    initial_content = "Hello World!"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://localhost:3000/api/documents `
    -Method POST `
    -Body $body `
    -ContentType "application/json" `
    -UseBasicParsing | Select-Object -ExpandProperty Content
```

Response (201 Created):
```json
{
  "id": "ef42531e-729d-465f-a23a-955dc96068e0",
  "user_id": "bb51bed5-28ed-4c59-b326-bfd628023c56",
  "title": "My First Document",
  "content": [1,246,146,248,193,12,12],
  "crdt_version": 0,
  "created_at": "2026-02-27T18:07:09.708174Z",
  "updated_at": "2026-02-27T18:07:09.708174Z"
}
```

### Get Document by ID

```powershell
Invoke-WebRequest -Uri http://localhost:3000/api/documents/ef42531e-729d-465f-a23a-955dc96068e0 `
    -Method GET `
    -UseBasicParsing | Select-Object -ExpandProperty Content
```

### List Documents (by User)

```powershell
Invoke-WebRequest -Uri "http://localhost:3000/api/documents?user_id=bb51bed5-28ed-4c59-b326-bfd628023c56" `
    -Method GET `
    -UseBasicParsing | Select-Object -ExpandProperty Content
```

### List All Documents (with pagination)

```powershell
Invoke-WebRequest -Uri "http://localhost:3000/api/documents?limit=10&offset=0" `
    -Method GET `
    -UseBasicParsing | Select-Object -ExpandProperty Content
```

### Update Document (Title Only)

```powershell
$body = @{
    title = "Updated Document Title"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://localhost:3000/api/documents/ef42531e-729d-465f-a23a-955dc96068e0 `
    -Method PUT `
    -Body $body `
    -ContentType "application/json" `
    -UseBasicParsing
```

### Delete Document

```powershell
Invoke-WebRequest -Uri http://localhost:3000/api/documents/ef42531e-729d-465f-a23a-955dc96068e0 `
    -Method DELETE `
    -UseBasicParsing | Select-Object -ExpandProperty Content
```

## WebSocket (Live Editing)

WebSocket endpoint: `ws://localhost:3000/ws/{document_id}`

Note: WebSocket implementation is pending (Task 7.1-7.4 in the spec).

## Database

- PostgreSQL (Aiven Cloud): Connected ✅
- Migrations: Completed ✅
- Tables: users, documents, presence

## What's Working

✅ User creation (no authentication)
✅ User retrieval and updates
✅ Document creation with CRDT initialization
✅ Document retrieval and listing
✅ Document updates (title)
✅ Document deletion
✅ PostgreSQL cloud storage
✅ CORS enabled for frontend
✅ Compression and tracing middleware

## What's Next

🔲 WebSocket live editing (Task 7.1-7.4)
🔲 CRDT sync engine (Task 8.1-8.4)
🔲 Offline support with SQLite (Task 12.1-12.5)
🔲 Real-time presence ("someone is typing...")
