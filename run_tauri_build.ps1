# AllBright DeFi - Production Tauri Build Script
# Builds MSI and NSIS installers for Windows deployment

$ErrorActionPreference = "Stop"
$Root = "D:\MS1\AB4"

# Add cargo to PATH
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
$env:PATH = "$cargoBin;" + $env:PATH

Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host " AllBright DeFi - Production Tauri Build (MSI + NSIS) " -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host ""

# Verify prerequisites
Write-Host "[PRE-CHECK] Verifying build prerequisites..." -ForegroundColor Yellow

# Check frontend build
$distPath = Join-Path $Root "apps\dashboard\dist\index.html"
if (-Not (Test-Path $distPath)) {
    Write-Host "[ERROR] Frontend not built. Run npm build first." -ForegroundColor Red
    exit 1
}
Write-Host "  - Frontend build verified: OK" -ForegroundColor Green

# Check cargo
$cargoPath = Join-Path $cargoBin "cargo.exe"
if (-Not (Test-Path $cargoPath)) {
    Write-Host "[ERROR] Cargo not found at $cargoPath" -ForegroundColor Red
    Write-Host "  Please install Rust from https://rustup.rs" -ForegroundColor Yellow
    exit 1
}
Write-Host "  - Cargo found: OK" -ForegroundColor Green

# Check tauri CLI
$tauriPath = Join-Path $cargoBin "tauri.exe"
if (-Not (Test-Path $tauriPath)) {
    Write-Host "[WARNING] Tauri CLI not found. Installing..." -ForegroundColor Yellow
    & cargo install tauri-cli --version "2"
}
Write-Host "  - Tauri CLI verified: OK" -ForegroundColor Green

# Check icon
$iconPath = Join-Path $Root "icons\icon.ico"
if (-Not (Test-Path $iconPath)) {
    Write-Host "[WARNING] Icon not found at $iconPath - using default" -ForegroundColor Yellow
} else {
    Write-Host "  - Application icon: OK" -ForegroundColor Green
}

Write-Host ""
Write-Host "[BUILD] Starting Tauri MSI + NSIS build..." -ForegroundColor Yellow

# Run Tauri build
Set-Location (Join-Path $Root "src-tauri")
$logPath = Join-Path $Root "tauri_build.log"

& cargo tauri build --bundles msi,nsis *>> $logPath

if ($LASTEXITCODE -eq 0) {
    Write-Host "" -ForegroundColor Green
    Write-Host "=====================================================" -ForegroundColor Green
    Write-Host " BUILD SUCCESSFUL " -ForegroundColor Green
    Write-Host "=====================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Output files:" -ForegroundColor Cyan
    Get-ChildItem "target\release\bundle" -Recurse -Include *.msi,*.exe 2>$null | ForEach-Object {
        $size = [math]::Round($_.Length/1MB, 2)
        Write-Host "  $($_.Name) ($size MB)" -ForegroundColor White
    }
} else {
    Write-Host "" -ForegroundColor Red
    Write-Host "=====================================================" -ForegroundColor Red
    Write-Host " BUILD FAILED " -ForegroundColor Red
    Write-Host "=====================================================" -ForegroundColor Red
    Write-Host "" -ForegroundColor Red
    Write-Host "Check log: $logPath" -ForegroundColor Yellow
    exit 1
}