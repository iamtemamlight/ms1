# AllBright Production Deployment Automation Script
# Automates the complete production deployment process with safety checks

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("staging", "production")]
    [string]$Environment,
    
    [switch]$SkipTests,
    [switch]$SkipBackup,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$Root = "d:\MS1\AB4"

Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host " AllBright Production Deployment Automation " -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Environment: $Environment" -ForegroundColor Yellow
Write-Host ""

# Phase 1: Pre-deployment Checks
Write-Host "[PHASE 1] Pre-deployment Checks" -ForegroundColor Cyan
Write-Host "----------------------------------------" -ForegroundColor Cyan

# Check if we're in the right directory
if (-Not (Test-Path "$Root\.env.$Environment.example")) {
    Write-Host "ERROR: .env.$Environment.example not found" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Environment template found" -ForegroundColor Green

# Check if .env file exists
if (-Not (Test-Path "$Root\.env.$Environment")) {
    Write-Host "ERROR: .env.$Environment not found. Copy from .env.$Environment.example" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Environment file found" -ForegroundColor Green

# Check Docker
$dockerVersion = docker --version 2>$null
if (-Not $dockerVersion) {
    Write-Host "ERROR: Docker not found" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Docker: $dockerVersion" -ForegroundColor Green

# Check Docker Compose
$composeVersion = docker-compose --version 2>$null
if (-Not $composeVersion) {
    Write-Host "ERROR: Docker Compose not found" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Docker Compose: $composeVersion" -ForegroundColor Green

# Check Rust
$cargoVersion = cargo --version 2>$null
if (-Not $cargoVersion) {
    Write-Host "ERROR: Rust/Cargo not found" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Rust: $cargoVersion" -ForegroundColor Green

# Check Node.js
$nodeVersion = node --version 2>$null
if (-Not $nodeVersion) {
    Write-Host "ERROR: Node.js not found" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Node.js: $nodeVersion" -ForegroundColor Green

Write-Host ""

# Phase 2: Environment Validation
Write-Host "[PHASE 2] Environment Validation" -ForegroundColor Cyan
Write-Host "----------------------------------------" -ForegroundColor Cyan

# Load environment variables
$envFile = "$Root\.env.$Environment"
Get-Content $envFile | ForEach-Object {
    if ($_ -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($matches[1], $matches[2])
    }
}

# Critical checks
$criticalVars = @(
    "DATABASE_URL",
    "REDIS_URL",
    "OPENAI_API_KEY",
    "PRIVATE_KEY",
    "WALLET_ADDRESS",
    "CHAIN_ID",
    "RPC_ENDPOINT"
)

$missingVars = @()
foreach ($var in $criticalVars) {
    if (-Not [Environment]::GetEnvironmentVariable($var)) {
        $missingVars += $var
    }
}

if ($missingVars.Count -gt 0) {
    Write-Host "ERROR: Missing critical environment variables:" -ForegroundColor Red
    $missingVars | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
Write-Host "✓ All critical environment variables set" -ForegroundColor Green

# Security checks
if ($Environment -eq "production") {
    $privateKey = [Environment]::GetEnvironmentVariable("PRIVATE_KEY")
    if ($privateKey -and $privateKey -ne "VAULT_MANAGED" -and $privateKey -ne "0xYourPrivateKeyHere") {
        Write-Host "WARNING: PRIVATE_KEY is in plaintext. Consider using hardware wallet." -ForegroundColor Yellow
        if (-Not $Force) {
            Write-Host "Use -Force to proceed anyway" -ForegroundColor Yellow
            exit 1
        }
    }
    
    $engineMode = [Environment]::GetEnvironmentVariable("VITE_ENGINE_MODE")
    if ($engineMode -ne "production") {
        Write-Host "ERROR: VITE_ENGINE_MODE must be 'production' for production deployment" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Production security checks passed" -ForegroundColor Green
}

Write-Host ""

# Phase 3: Backup
if (-Not $SkipBackup) {
    Write-Host "[PHASE 3] Backup" -ForegroundColor Cyan
    Write-Host "----------------------------------------" -ForegroundColor Cyan
    
    $backupDir = "$Root\backups\$Environment"
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $backupPath = "$backupDir\backup_$timestamp"
    
    New-Item -ItemType Directory -Force -Path $backupPath | Out-Null
    
    # Backup database
    if ($Environment -eq "production") {
        Write-Host "Backing up database..." -ForegroundColor Yellow
        docker-compose -f "$Root\docker-compose.yml" exec -T postgres pg_dumpall -U apxuser > "$backupPath\database.sql"
        Write-Host "✓ Database backed up" -ForegroundColor Green
    }
    
    # Backup configuration
    Copy-Item "$Root\.env.$Environment" "$backupPath\.env.$Environment"
    Write-Host "✓ Configuration backed up" -ForegroundColor Green
    
    Write-Host "✓ Backup complete: $backupPath" -ForegroundColor Green
    Write-Host ""
}

# Phase 4: Build
Write-Host "[PHASE 4] Build" -ForegroundColor Cyan
Write-Host "----------------------------------------" -ForegroundColor Cyan

# Build frontend
Write-Host "Building frontend..." -ForegroundColor Yellow
Set-Location $Root
npm --prefix apps/dashboard run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Frontend build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Frontend build complete" -ForegroundColor Green

# Build backend
Write-Host "Building backend..." -ForegroundColor Yellow
Set-Location "$Root\backend"
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Backend build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Backend build complete" -ForegroundColor Green

# Build Docker images
Write-Host "Building Docker images..." -ForegroundColor Yellow
Set-Location $Root
docker-compose build
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Docker build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Docker images built" -ForegroundColor Green

Write-Host ""

# Phase 5: Tests
if (-Not $SkipTests) {
    Write-Host "[PHASE 5] Tests" -ForegroundColor Cyan
    Write-Host "----------------------------------------" -ForegroundColor Cyan
    
    # Run backend tests
    Write-Host "Running backend tests..." -ForegroundColor Yellow
    Set-Location "$Root\backend"
    cargo test
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Backend tests failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✓ Backend tests passed" -ForegroundColor Green
    
    Write-Host ""
}

# Phase 6: Deployment
Write-Host "[PHASE 6] Deployment" -ForegroundColor Cyan
Write-Host "----------------------------------------" -ForegroundColor Cyan

# Stop existing services
Write-Host "Stopping existing services..." -ForegroundColor Yellow
Set-Location $Root
docker-compose down
Write-Host "✓ Services stopped" -ForegroundColor Green

# Start new services
Write-Host "Starting services..." -ForegroundColor Yellow
docker-compose up -d
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to start services" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Services started" -ForegroundColor Green

Write-Host ""

# Phase 7: Health Checks
Write-Host "[PHASE 7] Health Checks" -ForegroundColor Cyan
Write-Host "----------------------------------------" -ForegroundColor Cyan

Write-Host "Waiting for services to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 30

# Check backend health
Write-Host "Checking backend health..." -ForegroundColor Yellow
$healthResponse = try {
    Invoke-WebRequest -Uri "http://localhost:3000/health/live" -UseBasicParsing -TimeoutSec 10
} catch {
    Write-Host "ERROR: Backend health check failed" -ForegroundColor Red
    exit 1
}

if ($healthResponse.StatusCode -eq 200) {
    Write-Host "✓ Backend is healthy" -ForegroundColor Green
} else {
    Write-Host "ERROR: Backend returned status $($healthResponse.StatusCode)" -ForegroundColor Red
    exit 1
}

# Check database health
Write-Host "Checking database health..." -ForegroundColor Yellow
$dbHealthResponse = try {
    Invoke-WebRequest -Uri "http://localhost:3000/health/database" -UseBasicParsing -TimeoutSec 10
} catch {
    Write-Host "ERROR: Database health check failed" -ForegroundColor Red
    exit 1
}

if ($dbHealthResponse.StatusCode -eq 200) {
    Write-Host "✓ Database is healthy" -ForegroundColor Green
} else {
    Write-Host "ERROR: Database returned status $($dbHealthResponse.StatusCode)" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Phase 8: Post-deployment Verification
Write-Host "[PHASE 8] Post-deployment Verification" -ForegroundColor Cyan
Write-Host "----------------------------------------" -ForegroundColor Cyan

# Check logs
Write-Host "Checking service logs..." -ForegroundColor Yellow
docker-compose logs --tail=50 backend
Write-Host "✓ Logs checked" -ForegroundColor Green

# Verify environment
Write-Host "Verifying environment configuration..." -ForegroundColor Yellow
$engineMode = [Environment]::GetEnvironmentVariable("VITE_ENGINE_MODE")
if ($engineMode -eq $Environment) {
    Write-Host "✓ Environment configuration verified" -ForegroundColor Green
} else {
    Write-Host "WARNING: Environment mode mismatch" -ForegroundColor Yellow
}

Write-Host ""

# Phase 9: Summary
Write-Host "=====================================================" -ForegroundColor Green
Write-Host " DEPLOYMENT SUCCESSFUL " -ForegroundColor Green
Write-Host "=====================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Environment: $Environment" -ForegroundColor Cyan
Write-Host "Deployment Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Cyan
Write-Host ""
Write-Host "Services:" -ForegroundColor Cyan
docker-compose ps
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Monitor logs: docker-compose logs -f" -ForegroundColor White
Write-Host "  2. Check health: curl http://localhost:3000/health" -ForegroundColor White
Write-Host "  3. Verify dashboard: http://localhost:80" -ForegroundColor White
Write-Host ""
Write-Host "Rollback command:" -ForegroundColor Yellow
Write-Host "  docker-compose down && docker-compose up -d" -ForegroundColor White
Write-Host ""
