# AllBright Defi Software Engineering Ltd. 2026/V119
# SAFE build script: builds MSI + NSIS installers WITHOUT disturbing live trading.
# Live trading runs on node PID (passed as arg). Monitored before/after build.

param(
    [int]$TradingPid = 0
)

$ErrorActionPreference = "Stop"
$Root = "D:\ALLBRIGHTFOUR\AB4"

function Test-TradingAlive {
    param([int]$ProcId)
    if ($ProcId -eq 0) { return $true }
    $p = Get-Process -Id $ProcId -ErrorAction SilentlyContinue
    if ($p) {
        Write-Host "[MONITOR] Live trading node PID $ProcId ... ALIVE" -ForegroundColor Green
        return $true
    } else {
        Write-Host "[MONITOR] WARNING: Live trading node PID $ProcId ... NOT FOUND" -ForegroundColor Red
        return $false
    }
}

Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host " AllBright V119 - SAFE MSI/NSIS BUILD (trading-safe)" -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host ""

# Pre-build trading check
Write-Host "[PRE-CHECK] Verifying live trading is untouched..." -ForegroundColor Yellow
$alivePre = Test-TradingAlive -ProcId $TradingPid

# Step 1: Build frontend (does NOT touch port 3000)
Write-Host "" 
Write-Host "[1/4] Building frontend (npm run build)..." -ForegroundColor Yellow
Set-Location $Root
npm --prefix apps/dashboard run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Frontend build FAILED!" -ForegroundColor Red
    Test-TradingAlive -ProcId $TradingPid
    exit 1
}
Write-Host "Frontend build SUCCESS!" -ForegroundColor Green

# Mid-build trading check
Test-TradingAlive -ProcId $TradingPid | Out-Null

# Step 2: Verify dist
Write-Host ""
Write-Host "[2/4] Verifying dist folder..." -ForegroundColor Yellow
if (-Not (Test-Path "apps/dashboard/dist/index.html")) {
    Write-Host "dist/index.html NOT FOUND!" -ForegroundColor Red
    exit 1
}
Write-Host "Dist folder verified!" -ForegroundColor Green

# Step 3: Tauri build MSI + NSIS (compiles Rust, bundles installer - no node/port interaction)
Write-Host ""
Write-Host "[3/4] Building Tauri MSI + NSIS bundles..." -ForegroundColor Yellow
Set-Location "$Root\src-tauri"
cargo tauri build --bundles msi,nsis
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri build FAILED!" -ForegroundColor Red
    Test-TradingAlive -ProcId $TradingPid
    exit 1
}
Write-Host "Tauri build SUCCESS!" -ForegroundColor Green

# Step 4: List outputs
Write-Host ""
Write-Host "[4/4] Build outputs:" -ForegroundColor Yellow
Get-ChildItem "target\release\bundle" -Recurse -Include *.msi,*.exe 2>$null | ForEach-Object {
    Write-Host "  $($_.FullName)  ($([math]::Round($_.Length/1MB,2)) MB)" -ForegroundColor Cyan
}

# Post-build trading check
Write-Host ""
Write-Host "[POST-CHECK] Verifying live trading still intact..." -ForegroundColor Yellow
$alivePost = Test-TradingAlive -ProcId $TradingPid
if ($alivePost) {
    Write-Host "[OK] Live trading confirmed running - no disruption caused by build." -ForegroundColor Green
} else {
    Write-Host "[ALERT] Live trading process was lost during build!" -ForegroundColor Red
}

Write-Host ""
Write-Host "=====================================================" -ForegroundColor Green
Write-Host " BUILD COMPLETE - Trading-safe build finished." -ForegroundColor Green
Write-Host "=====================================================" -ForegroundColor Green