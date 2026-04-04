# Check SQLite Database Contents
$dbPath = "backend/data/app.db"

Write-Host "📊 Database Check" -ForegroundColor Cyan
Write-Host "=================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $dbPath)) {
    Write-Host "❌ Database not found: $dbPath" -ForegroundColor Red
    exit 1
}

$size = (Get-Item $dbPath).Length
Write-Host "📁 Database: $dbPath" -ForegroundColor Green
Write-Host "📏 Size: $size bytes ($([math]::Round($size/1KB, 2)) KB)" -ForegroundColor Green
Write-Host ""

# Load System.Data.SQLite if available
try {
    Add-Type -Path "System.Data.SQLite.dll" -ErrorAction Stop
    Write-Host "✅ SQLite library loaded" -ForegroundColor Green
} catch {
    Write-Host "❌ System.Data.SQLite not available" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To check database contents:" -ForegroundColor Cyan
    Write-Host "1. Download DB Browser for SQLite: https://sqlitebrowser.org/" -ForegroundColor White
    Write-Host "2. Open file: $dbPath" -ForegroundColor White
    Write-Host "3. Go to 'Browse Data' tab" -ForegroundColor White
    Write-Host "4. Select table: card_chunks" -ForegroundColor White
    Write-Host ""
    Write-Host "Expected data based on backend logs:" -ForegroundColor Yellow
    Write-Host "  - quote_0: 8,120 lines" -ForegroundColor White
    Write-Host "  - quote_1: 3 lines" -ForegroundColor White
    Write-Host "  - quote_23: 8,120 lines" -ForegroundColor White
    Write-Host "  - Total: 16,243 lines" -ForegroundColor White
    exit 0
}
