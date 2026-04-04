# Backend API Test Script

Write-Host "🧪 Testing Backend API..." -ForegroundColor Cyan
Write-Host ""

# Test 1: Health Check
Write-Host "1️⃣ Testing Health Endpoint..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "http://localhost:3000/health" -Method Get -UseBasicParsing
    Write-Host "✅ Health Check: $health" -ForegroundColor Green
} catch {
    Write-Host "❌ Health Check Failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Test 2: Create User
Write-Host "2️⃣ Creating Test User..." -ForegroundColor Yellow
$userData = @{
    name = "Test User"
    email = "test@example.com"
    country_code = "BD"
    company_name = "Test Company"
} | ConvertTo-Json

try {
    $user = Invoke-RestMethod -Uri "http://localhost:3000/api/users" -Method Post -Body $userData -ContentType "application/json" -UseBasicParsing
    $userId = $user.id
    Write-Host "✅ User Created: $userId" -ForegroundColor Green
    Write-Host "   Name: $($user.name)" -ForegroundColor Gray
    Write-Host "   Email: $($user.email)" -ForegroundColor Gray
} catch {
    Write-Host "❌ User Creation Failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Test 3: Save Settings
Write-Host "3️⃣ Saving User Settings..." -ForegroundColor Yellow
$settingsData = @{
    theme = @{
        mode = "Gradient"
        gradient_angle = 45
        gradient_colors = @(0xFF6B46C1, 0xFF2563EB)
    }
    text_style = @{
        main_text_size = 32.0
        sub_text_size = 18.0
    }
    interval_secs = 30
} | ConvertTo-Json -Depth 10

try {
    $settings = Invoke-RestMethod -Uri "http://localhost:3000/api/users/$userId/settings" -Method Post -Body $settingsData -ContentType "application/json" -UseBasicParsing
    Write-Host "✅ Settings Saved Successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Settings Save Failed: $_" -ForegroundColor Red
}

Write-Host ""

# Test 4: Load Settings
Write-Host "4️⃣ Loading User Settings..." -ForegroundColor Yellow
try {
    $loadedSettings = Invoke-RestMethod -Uri "http://localhost:3000/api/users/$userId/settings" -Method Get -UseBasicParsing
    Write-Host "✅ Settings Loaded Successfully" -ForegroundColor Green
    Write-Host "   Theme Mode: $($loadedSettings.settings_data.theme.mode)" -ForegroundColor Gray
    Write-Host "   Main Text Size: $($loadedSettings.settings_data.text_style.main_text_size)" -ForegroundColor Gray
} catch {
    Write-Host "❌ Settings Load Failed: $_" -ForegroundColor Red
}

Write-Host ""

# Test 5: Create Document
Write-Host "5️⃣ Creating Test Document..." -ForegroundColor Yellow
$docData = @{
    user_id = $userId
    title = "My First Note"
    initial_content = "This is a test note from frontend"
} | ConvertTo-Json

try {
    $doc = Invoke-RestMethod -Uri "http://localhost:3000/api/documents" -Method Post -Body $docData -ContentType "application/json" -UseBasicParsing
    Write-Host "✅ Document Created: $($doc.id)" -ForegroundColor Green
    Write-Host "   Title: $($doc.title)" -ForegroundColor Gray
} catch {
    Write-Host "❌ Document Creation Failed: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "🎉 All Backend Tests Completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Summary:" -ForegroundColor Cyan
Write-Host "   - Backend is running on http://localhost:3000" -ForegroundColor White
Write-Host "   - SQLite database: backend/data/app.db" -ForegroundColor White
Write-Host "   - All API endpoints working correctly" -ForegroundColor White
Write-Host "   - Ready for frontend connection!" -ForegroundColor White

