# Setup Kiro Agent - After Git Clone
# This script helps configure Kiro with project-specific settings

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Kiro Agent Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Kiro is installed
$kiroPath = "$env:APPDATA\Kiro\argv.json"
if (-not (Test-Path $kiroPath)) {
    Write-Host "❌ Kiro not found at: $kiroPath" -ForegroundColor Red
    Write-Host "Please install Kiro first from https://kiro.so" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Kiro found at: $kiroPath" -ForegroundColor Green
Write-Host ""

# Read current argv.json
try {
    $currentConfig = Get-Content $kiroPath -Raw | ConvertFrom-Json
} catch {
    Write-Host "⚠️  Could not read current Kiro config" -ForegroundColor Yellow
    $currentConfig = New-Object PSObject
}

# Create new trusted commands array
$trustedCommands = @()

# Add project-specific trusted commands
Write-Host "Adding project trusted commands..." -ForegroundColor Yellow

$trustedCommands += @{
    name = "cargo-check"
    command = "cargo check"
    description = "Run cargo check on the project"
}

$trustedCommands += @{
    name = "cargo-build-release"
    command = "cargo build --release"
    description = "Build the project in release mode"
}

$trustedCommands += @{
    name = "cargo-test"
    command = "cargo test"
    description = "Run tests"
}

$trustedCommands += @{
    name = "backend-start"
    command = "cd backend; cargo run --release"
    description = "Start the backend API server"
}

$trustedCommands += @{
    name = "frontend-start"
    command = "cargo run --release"
    description = "Start the desktop app"
}

# Update config
if (-not $currentConfig.trustedCommands) {
    $currentConfig | Add-Member -MemberType NoteProperty -Name "trustedCommands" -Value @()
}

# Merge commands (avoid duplicates)
$existingNames = $currentConfig.trustedCommands.name
foreach ($cmd in $trustedCommands) {
    if ($cmd.name -notin $existingNames) {
        $currentConfig.trustedCommands += $cmd
        Write-Host "  ✅ Added: $($cmd.name)" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  Skipped (already exists): $($cmd.name)" -ForegroundColor Yellow
    }
}

# Save updated config
try {
    $currentConfig | ConvertTo-Json -Depth 10 | Set-Content $kiroPath
    Write-Host ""
    Write-Host "✅ Kiro setup complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "You can now use these commands in Kiro:" -ForegroundColor Cyan
    foreach ($cmd in $currentConfig.trustedCommands) {
        Write-Host "  - $($cmd.name): $($cmd.description)" -ForegroundColor White
    }
} catch {
    Write-Host "❌ Failed to update Kiro config: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Setup Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
