# AllBright DeFi - Production Preflight Verification Script
# Verifies all configurations before live trading deployment

$ErrorActionPreference = "Stop"
$Root = "D:\MS1\AB4"

Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host " AllBright DeFi - Production Preflight Verification " -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host ""

$errors = 0
$warnings = 0

# 1. Check Environment Configuration
Write-Host "[1/5] Checking environment configuration..." -ForegroundColor Yellow

$envFile = Join-Path $Root ".env"
if (Test-Path $envFile) {
    $content = Get-Content $envFile -Raw
    
    # Check AUTO_TRANSFER_ENABLED=false
    if ($content -match "AUTO_TRANSFER_ENABLED=false") {
        Write-Host "  - Auto-transfer DISABLED: OK" -ForegroundColor Green
    } else {
        Write-Host "  - Auto-transfer DISABLED: MISSING" -ForegroundColor Red
        $errors++
    }
    
    # Check 10 ETH threshold
    if ($content -match "AUTO_TRANSFER_THRESHOLD_ETH=10.0") {
        Write-Host "  - 10 ETH profit threshold: OK" -ForegroundColor Green
    } else {
        Write-Host "  - 10 ETH profit threshold: MISSING" -ForegroundColor Yellow
        $warnings++
    }
    
    # Check production mode
    if ($content -match "VITE_ENGINE_MODE=production") {
        Write-Host "  - Production mode enabled: OK" -ForegroundColor Green
    } else {
        Write-Host "  - Production mode enabled: MISSING" -ForegroundColor Red
        $errors++
    }
    
    # Check demo mode disabled
    if ($content -match "VITE_DEMO_MODE=false") {
        Write-Host "  - Demo mode disabled: OK" -ForegroundColor Green
    } else {
        Write-Host "  - Demo mode disabled: WARNING (may block live trading)" -ForegroundColor Yellow
        $warnings++
    }
} else {
    Write-Host "  - ERROR: .env file not found!" -ForegroundColor Red
    $errors++
}

Write-Host ""

# 2. Check Backend Configuration
Write-Host "[2/5] Checking backend configuration..." -ForegroundColor Yellow

$backendEnv = Join-Path $Root "backend\.env"
if (Test-Path $backendEnv) {
    $content = Get-Content $backendEnv -Raw
    
    if ($content -match "AUTO_TRANSFER_ENABLED=false") {
        Write-Host "  - Backend auto-transfer DISABLED: OK" -ForegroundColor Green
    } else {
        Write-Host "  - Backend auto-transfer DISABLED: CONFIGURED (check value)" -ForegroundColor Yellow
        $warnings++
    }
    
    if ($content -match "VITE_ENGINE_MODE=production") {
        Write-Host "  - Backend production mode: OK" -ForegroundColor Green
    } else {
        Write-Host "  - Backend production mode: CHECK VALUE" -ForegroundColor Yellow
    }
} else {
    Write-Host "  - WARNING: backend\.env not found" -ForegroundColor Yellow
    $warnings++
}

Write-Host ""

# 3. Check Frontend Build
Write-Host "[3/5] Checking frontend build..." -ForegroundColor Yellow

$distPath = Join-Path $Root "apps\dashboard\dist\index.html"
if (Test-Path $distPath) {
    Write-Host "  - Frontend dist exists: OK" -ForegroundColor Green
} else {
    Write-Host "  - Frontend dist not found: Run npm build first!" -ForegroundColor Red
    $errors++
}

$walletView = Join-Path $Root "apps\dashboard\src\components\WalletView.tsx"
if (Test-Path $walletView) {
    $content = Get-Content $walletView -Raw
    if ($content -match "executeManualTransfer") {
        Write-Host "  - Manual transfer function: OK" -ForegroundColor Green
    } else {
        Write-Host "  - Manual transfer function: MISSING" -ForegroundColor Red
        $errors++
    }
} else {
    Write-Host "  - WalletView.tsx not found" -ForegroundColor Red
    $errors++
}

Write-Host ""

# 4. Check Tauri Configuration
Write-Host "[4/5] Checking Tauri configuration..." -ForegroundColor Yellow

$tauriConf = Join-Path $Root "src-tauri\tauri.conf.json"
if (Test-Path $tauriConf) {
    $content = Get-Content $tauriConf -Raw
    if ($content -match "msi" -and $content -match "nsis") {
        Write-Host "  - MSI/NSIS targets configured: OK" -ForegroundColor Green
    } else {
        Write-Host "  - MSI/NSIS targets configured: MISSING" -ForegroundColor Red
        $errors++
    }
} else {
    Write-Host "  - tauri.conf.json not found" -ForegroundColor Red
    $errors++
}

$tauriCargo = Join-Path $Root "src-tauri\Cargo.toml"
if (Test-Path $tauriCargo) {
    Write-Host "  - Cargo.toml exists: OK" -ForegroundColor Green
} else {
    Write-Host "  - Cargo.toml not found" -ForegroundColor Red
    $errors++
}

Write-Host ""

# 5. Check Security Configuration
Write-Host "[5/5] Checking security configuration..." -ForegroundColor Yellow

$encryptScript = Join-Path $Root "scripts\encrypt_env_files.py"
if (Test-Path $encryptScript) {
    Write-Host "  - Encryption script available: OK" -ForegroundColor Green
} else {
    Write-Host "  - Encryption script: MISSING" -ForegroundColor Yellow
    $warnings++
}

# Check private key exposure in frontend (critical)
$walletViewContent = Get-Content $walletView -Raw
if ($walletViewContent -match "VAULT_MANAGED") {
    Write-Host "  - Private key security (vault placeholder): OK" -ForegroundColor Green
} else {
    Write-Host "  - Private key security: WARNING" -ForegroundColor Yellow
    $warnings++
}

Write-Host ""
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host " VERIFICATION SUMMARY " -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan

if ($errors -gt 0) {
    Write-Host "Errors: $errors" -ForegroundColor Red
} else {
    Write-Host "Errors: 0" -ForegroundColor Green
}

if ($warnings -gt 0) {
    Write-Host "Warnings: $warnings" -ForegroundColor Yellow
} else {
    Write-Host "Warnings: 0" -ForegroundColor Green
}

Write-Host ""

if ($errors -eq 0) {
    Write-Host "✅ SYSTEM READY FOR PRODUCTION DEPLOYMENT" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor White
    Write-Host "  1. Run: .\\run_tauri_build.ps1 (to build MSI/NSIS installers)" -ForegroundColor White
    Write-Host "  2. Install the generated .msi or .exe installer" -ForegroundColor White
    Write-Host "  3. Monitor Smart Vault for ≥ 10 ETH profit threshold" -ForegroundColor White
    Write-Host "  4. Execute manual transfer via WalletView when threshold met" -ForegroundColor White
} else {
    Write-Host "❌ SYSTEM NOT READY - FIX ERRORS BEFORE DEPLOYMENT" -ForegroundColor Red
    exit 1
}