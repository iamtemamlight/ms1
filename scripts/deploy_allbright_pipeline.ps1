<#
.SYNOPSIS
    AllBright Deployment Pipeline — Preflight → Simulation → Live
    Automates the full copilot deployment workflow with autonomous error fixing.

.DESCRIPTION
    This script drives the AllBright system through the complete deployment pipeline:
    1. Authorize copilot in the selected mode (manual/assisted/autonomous)
    2. Run preflight checks (auto-fixes logging/env/rpc/db errors in Autonomous mode)
    3. Run simulation (auto-fixes logging/arbitrage/risk errors in Autonomous mode)
    4. Transform to live production (auto-fixes logging/deploy/auth errors in Autonomous mode)
    
    In Autonomous mode, the copilot self-diagnoses and fixes ALL logging-system errors
    (LOG_BUFFER_OVERFLOW, LOG_WRITER_PANIC, LOG_SINK_DISCONNECTED, LOG_ROTATION_FAILED,
     LOG_SERIALIZATION_ERROR, LOG_PERMISSION_DENIED, LOG_DISK_FULL) without calling the Commander.

.PARAMETER Mode
    Deployment mode: "manual", "assisted", or "autonomous" (default: autonomous)

.PARAMETER BackendUrl
    Backend API base URL (default: http://localhost:50051)

.PARAMETER AutoFix
    If set, automatically attempt fixes for all errors (only effective in autonomous mode)

.EXAMPLE
    .\deploy_allbright_pipeline.ps1 -Mode autonomous
    .\deploy_allbright_pipeline.ps1 -Mode assisted -BackendUrl http://localhost:50051
#>

param(
    [ValidateSet("manual", "assisted", "autonomous")]
    [string]$Mode = "autonomous",
    [string]$BackendUrl = "http://localhost:50051",
    [switch]$AutoFix = $true
)

$ErrorActionPreference = "Stop"
$api = "$BackendUrl/api/deployment"

Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     ALLBRIGHT DEPLOYMENT PIPELINE (Preflight→Simulation→Live)    ║" -ForegroundColor Cyan
Write-Host "║     Mode: $($Mode.ToUpper())                                          ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── Step 0: Health check ──
Write-Host "» Checking backend health..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$BackendUrl/healthz" -TimeoutSec 5
    Write-Host "  ✓ Backend is healthy" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Backend unreachable at $BackendUrl — is the server running?" -ForegroundColor Red
    exit 1
}

# ── Step 1: Authorize ──
Write-Host "`n» Step 1/4: Authorizing copilot in $Mode mode..." -ForegroundColor Yellow
try {
    $auth = Invoke-RestMethod -Uri "$api/authorize" -Method Post `
        -ContentType "application/json" `
        -Body (ConvertTo-Json @{ mode = $Mode }) `
        -TimeoutSec 10
    Write-Host "  ✓ Authorized (stage: $($auth.current_stage))" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Authorization failed: $_" -ForegroundColor Red
    exit 1
}

# ── Step 2: Preflight ──
Write-Host "`n» Step 2/4: Running preflight checks..." -ForegroundColor Yellow
try {
    $preflight = Invoke-RestMethod -Uri "$api/preflight" -Method Post -TimeoutSec 30
    Write-Host "  ✓ Preflight completed (stage: $($preflight.current_stage))" -ForegroundColor Green
    
    # Check for errors and auto-fix if in autonomous mode
    $errors = $preflight.errors | Where-Object { $_.stage -eq "preflight" -and -not $_.fixed }
    if ($errors.Count -gt 0) {
        Write-Host "  ⚠ $($errors.Count) preflight error(s) detected" -ForegroundColor Yellow
        foreach ($err in $errors) {
            if ($Mode -eq "autonomous" -and $AutoFix) {
                Write-Host "    → Auto-fixing $($err.code)..." -ForegroundColor Cyan
                $fix = Invoke-RestMethod -Uri "$api/log-diagnose" -Method Post `
                    -ContentType "application/json" `
                    -Body (ConvertTo-Json @{ stage = "preflight"; error_code = $err.code; message = $err.message }) `
                    -TimeoutSec 10
                if ($fix.fixed) {
                    Write-Host "    ✓ Fixed: $($fix.fix)" -ForegroundColor Green
                } else {
                    Write-Host "    ✗ Could not auto-fix $($err.code)" -ForegroundColor Red
                }
            } else {
                Write-Host "    ⚠ $($err.code): $($err.message) (requires Commander in $Mode mode)" -ForegroundColor Yellow
            }
        }
    }
} catch {
    Write-Host "  ✗ Preflight failed: $_" -ForegroundColor Red
    exit 1
}

# ── Step 3: Simulation ──
Write-Host "`n» Step 3/4: Running simulation..." -ForegroundColor Yellow
try {
    $sim = Invoke-RestMethod -Uri "$api/simulation" -Method Post -TimeoutSec 30
    Write-Host "  ✓ Simulation completed (stage: $($sim.current_stage))" -ForegroundColor Green
    
    $simErrors = $sim.errors | Where-Object { $_.stage -eq "simulation" -and -not $_.fixed }
    if ($simErrors.Count -gt 0) {
        Write-Host "  ⚠ $($simErrors.Count) simulation error(s) detected" -ForegroundColor Yellow
        foreach ($err in $simErrors) {
            if ($Mode -eq "autonomous" -and $AutoFix) {
                Write-Host "    → Auto-fixing $($err.code)..." -ForegroundColor Cyan
                $fix = Invoke-RestMethod -Uri "$api/log-diagnose" -Method Post `
                    -ContentType "application/json" `
                    -Body (ConvertTo-Json @{ stage = "simulation"; error_code = $err.code; message = $err.message }) `
                    -TimeoutSec 10
                if ($fix.fixed) {
                    Write-Host "    ✓ Fixed: $($fix.fix)" -ForegroundColor Green
                } else {
                    Write-Host "    ✗ Could not auto-fix $($err.code)" -ForegroundColor Red
                }
            } else {
                Write-Host "    ⚠ $($err.code): $($err.message) (requires Commander in $Mode mode)" -ForegroundColor Yellow
            }
        }
    }
} catch {
    Write-Host "  ✗ Simulation failed: $_" -ForegroundColor Red
    exit 1
}

# ── Step 4: Transform to Live ──
Write-Host "`n» Step 4/4: Transforming to live production..." -ForegroundColor Yellow
try {
    $live = Invoke-RestMethod -Uri "$api/live" -Method Post -TimeoutSec 30
    Write-Host "  ✓ Live deployment completed (stage: $($live.current_stage))" -ForegroundColor Green
    
    $liveErrors = $live.errors | Where-Object { $_.stage -eq "live" -and -not $_.fixed }
    if ($liveErrors.Count -gt 0) {
        Write-Host "  ⚠ $($liveErrors.Count) live deployment error(s) detected" -ForegroundColor Yellow
        foreach ($err in $liveErrors) {
            if ($Mode -eq "autonomous" -and $AutoFix) {
                Write-Host "    → Auto-fixing $($err.code)..." -ForegroundColor Cyan
                $fix = Invoke-RestMethod -Uri "$api/log-diagnose" -Method Post `
                    -ContentType "application/json" `
                    -Body (ConvertTo-Json @{ stage = "live"; error_code = $err.code; message = $err.message }) `
                    -TimeoutSec 10
                if ($fix.fixed) {
                    Write-Host "    ✓ Fixed: $($fix.fix)" -ForegroundColor Green
                } else {
                    Write-Host "    ✗ Could not auto-fix $($err.code)" -ForegroundColor Red
                }
            } else {
                Write-Host "    ⚠ $($err.code): $($err.message) (requires Commander in $Mode mode)" -ForegroundColor Yellow
            }
        }
    }
} catch {
    Write-Host "  ✗ Live deployment failed: $_" -ForegroundColor Red
    exit 1
}

# ── Summary ──
Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    DEPLOYMENT COMPLETE                      ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Fetch final status
try {
    $final = Invoke-RestMethod -Uri "$api/status" -TimeoutSec 5
    Write-Host "  Final stage: $($final.current_stage)" -ForegroundColor White
    Write-Host "  Progress: $($final.progress)%" -ForegroundColor White
    Write-Host "  Total errors: $($final.errors.Count)" -ForegroundColor White
    Write-Host "  Auto-fixed: $(($final.errors | Where-Object { $_.fixed }).Count)" -ForegroundColor Green
    Write-Host "  Log entries: $($final.logs.Count)" -ForegroundColor White
} catch {
    Write-Host "  (Status endpoint unavailable)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  Use: GET $api/logs to view full deployment log" -ForegroundColor Gray
Write-Host "  Use: POST $api/reset to reset deployment state" -ForegroundColor Gray