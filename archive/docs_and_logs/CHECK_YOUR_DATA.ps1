# PowerShell script to check your data in the backend

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Checking Your Backend Data" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Your user ID
$userId = "1cc88e68-896e-4fdb-abee-c27481343d83"

# 1. Check if backend is running
Write-Host "1. Checking backend health..." -ForegroundColor Yellow
try {
    $health = Invoke-WebRequest -Uri http://localhost:3000/health -Method GET -UseBasicParsing
    Write-Host "   ✅ Backend is running!" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Backend is not running. Start it with: cd backend && cargo run --release" -ForegroundColor Red
    exit
}
Write-Host ""

# 2. Get your user profile
Write-Host "2. Your User Profile:" -ForegroundColor Yellow
try {
    $user = Invoke-WebRequest -Uri "http://localhost:3000/api/users/$userId" -Method GET -UseBasicParsing
    $userJson = $user.Content | ConvertFrom-Json
    Write-Host "   Name:    $($userJson.name)" -ForegroundColor White
    Write-Host "   Email:   $($userJson.email)" -ForegroundColor White
    Write-Host "   Country: $($userJson.country_code)" -ForegroundColor White
    Write-Host "   Company: $($userJson.company_name)" -ForegroundColor White
    Write-Host "   ID:      $($userJson.id)" -ForegroundColor Gray
    Write-Host "   Created: $($userJson.created_at)" -ForegroundColor Gray
} catch {
    Write-Host "   ❌ Could not fetch user data" -ForegroundColor Red
}
Write-Host ""

# 3. List your documents
Write-Host "3. Your Documents/Quotes:" -ForegroundColor Yellow
try {
    $docs = Invoke-WebRequest -Uri "http://localhost:3000/api/documents?user_id=$userId" -Method GET -UseBasicParsing
    $docsJson = $docs.Content | ConvertFrom-Json
    
    if ($docsJson.Count -eq 0) {
        Write-Host "   No documents yet. Create one using the API!" -ForegroundColor Gray
    } else {
        foreach ($doc in $docsJson) {
            Write-Host "   📄 $($doc.title)" -ForegroundColor White
            Write-Host "      ID: $($doc.id)" -ForegroundColor Gray
            Write-Host "      Created: $($doc.created_at)" -ForegroundColor Gray
            Write-Host ""
        }
    }
} catch {
    Write-Host "   ❌ Could not fetch documents" -ForegroundColor Red
}
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  All data is stored in PostgreSQL!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
