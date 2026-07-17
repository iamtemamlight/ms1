# AllBright Defi Software Engineering Ltd. 2026/V119 - Build Script
# This script builds the frontend and creates MSI/NSIS installers.

$ErrorActionPreference = "Stop"
$Root = "D:\ALLBRIGHTFOUR\AB4"

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "AllBright Defi Software Engineering Ltd. 2026/V119 - Build Process Starting" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build the frontend
Write-Host "[1/4] Building frontend..." -ForegroundColor Yellow
Set-Location $Root
npm --prefix apps/dashboard run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Frontend build FAILED!" -ForegroundColor Red
    exit 1
}
Write-Host "Frontend build SUCCESS!" -ForegroundColor Green
Write-Host ""

# Step 2: Verify dist folder (the folder Tauri bundles into the installer)
Write-Host "[2/4] Verifying dist folder..." -ForegroundColor Yellow
if (-Not (Test-Path "apps/dashboard/dist/index.html")) {
    Write-Host "apps/dashboard/dist/index.html NOT FOUND!" -ForegroundColor Red
    exit 1
}
Write-Host "Dist folder verified!" -ForegroundColor Green
Write-Host ""

# Step 3: Build Tauri with MSI and NSIS
Write-Host "[3/4] Building Tauri applications (MSI + NSIS)..." -ForegroundColor Yellow
Set-Location "$Root\src-tauri"
cargo tauri build --bundles msi,nsis
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri build FAILED!" -ForegroundColor Red
    exit 1
}
Write-Host "Tauri build SUCCESS!" -ForegroundColor Green
Write-Host ""

# Step 4: List output files
Write-Host "[4/4] Build outputs:" -ForegroundColor Yellow
Get-ChildItem "target\release\bundle" -Recurse -Include *.msi,*.exe 2>$null | ForEach-Object {
    Write-Host "  $($_.FullName)" -ForegroundColor Cyan
}
Write-Host ""
Write-Host "=============================================" -ForegroundColor Green
Write-Host "BUILD COMPLETE!" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Green
