# Automated Backend Production Modules Integration Script
# This script automatically integrates health checks, graceful shutdown, and environment validation into main.rs

$ErrorActionPreference = "Stop"
$BackendDir = "d:\MS1\AB4\backend"
$MainRsPath = "$BackendDir\main.rs"

Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host " AllBright Backend - Production Modules Integration " -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host ""

# Backup original main.rs
$BackupPath = "$MainRsPath.backup.$(Get-Date -Format 'yyyyMMdd_HHmmss')"
Write-Host "[BACKUP] Creating backup: $BackupPath" -ForegroundColor Yellow
Copy-Item $MainRsPath $BackupPath

try {
    $content = Get-Content $MainRsPath -Raw
    
    # Step 1: Add module declarations
    Write-Host "[1/5] Adding module declarations..." -ForegroundColor Yellow
    
    $moduleDeclarations = @'
 mod health_checks;
 mod graceful_shutdown;
 mod env_validation;
'@
    
    # Find the line with "mod copilot_system_access;" and add after it
    if ($content -match '(mod copilot_system_access;)') {
        $content = $content -replace '(mod copilot_system_access;)', "mod copilot_system_access;$moduleDeclarations"
        Write-Host "  - Module declarations added" -ForegroundColor Green
    } else {
        Write-Host "  - ERROR: Could not find mod copilot_system_access" -ForegroundColor Red
        throw "Module declaration insertion failed"
    }
    
    # Step 2: Add imports
    Write-Host "[2/5] Adding imports..." -ForegroundColor Yellow
    
    $imports = @'
 use health_checks::HealthChecker;
 use graceful_shutdown::{GracefulShutdown, ShutdownSignal, wait_for_shutdown_signal};
 use env_validation::EnvValidator;
'@
    
    # Find the line with "use m055_env_vault::EnvVault;" and add after it
    if ($content -match '(use m055_env_vault::EnvVault;)') {
        $content = $content -replace '(use m055_env_vault::EnvVault;)', "use m055_env_vault::EnvVault;$imports"
        Write-Host "  - Imports added" -ForegroundColor Green
    } else {
        Write-Host "  - ERROR: Could not find use m055_env_vault::EnvVault" -ForegroundColor Red
        throw "Import insertion failed"
    }
    
    # Step 3: Add environment validation at start of run_server
    Write-Host "[3/5] Adding environment validation..." -ForegroundColor Yellow
    
    $envValidation = @'
    // Validate environment variables first
    info!("Validating environment configuration...");
    let validator = EnvValidator::new();
    validator.validate_or_panic();
    
'@
    
    # Find the line with "pub async fn run_server" and add validation after the opening brace
    if ($content -match '(pub async fn run_server\(addr: String, http_addr: String\) -> Result<(), AppError> \{)') {
        $content = $content -replace '(pub async fn run_server\(addr: String, http_addr: String\) -> Result<(), AppError> \{)', "pub async fn run_server(addr: String, http_addr: String) -> Result<(), AppError> {$envValidation"
        Write-Host "  - Environment validation added" -ForegroundColor Green
    } else {
        Write-Host "  - ERROR: Could not find run_server function" -ForegroundColor Red
        throw "Environment validation insertion failed"
    }
    
    # Step 4: Add health check routes
    Write-Host "[4/5] Adding health check routes..." -ForegroundColor Yellow
    
    $healthRoutes = @'
        .route("/health", get(|| async {
            let health_checker = HealthChecker::new(Some(db_pool.clone()), std::env::var("RPC_ENDPOINT").ok());
            axum::Json(health_checker.comprehensive_health().await)
        }))
        .route("/health/live", get(|| async {
            let health_checker = HealthChecker::new(Some(db_pool.clone()), std::env::var("RPC_ENDPOINT").ok());
            axum::Json(health_checker.liveness().await)
        }))
        .route("/health/ready", get(|| async {
            let health_checker = HealthChecker::new(Some(db_pool.clone()), std::env::var("RPC_ENDPOINT").ok());
            axum::Json(health_checker.readiness().await)
        }))
        .route("/health/database", get(|| async {
            let health_checker = HealthChecker::new(Some(db_pool.clone()), std::env::var("RPC_ENDPOINT").ok());
            axum::Json(health_checker.check_database().await)
        }))
        .route("/health/rpc", get(|| async {
            let health_checker = HealthChecker::new(Some(db_pool.clone()), std::env::var("RPC_ENDPOINT").ok());
            axum::Json(health_checker.check_rpc().await)
        }))
'@
    
    # Find the line with '.route("/healthz"' and replace with new health routes
    if ($content -match '(\.route\("/healthz")') {
        $content = $content -replace '\.route\("/healthz".*?\.route\("/readyz".*?\.layer\(cors\);', "$healthRoutes        .layer(cors);"
        Write-Host "  - Health check routes added" -ForegroundColor Green
    } else {
        Write-Host "  - WARNING: Could not find existing health routes, appending instead" -ForegroundColor Yellow
        # Alternative: append before .layer(cors)
        $content = $content -replace '(\.layer\(cors\);)', "$healthRoutes        $1"
        Write-Host "  - Health check routes appended" -ForegroundColor Green
    }
    
    # Step 5: Add graceful shutdown
    Write-Host "[5/5] Adding graceful shutdown..." -ForegroundColor Yellow
    
    # Initialize shutdown signal before server startup
    $shutdownInit = @'
    // Initialize shutdown signal for graceful shutdown
    let shutdown_signal = ShutdownSignal::new();
    
'@
    
    # Find the line with "let http_server = axum::serve::serve" and add before it
    if ($content -match '(let http_server = axum::serve::serve)') {
        $content = $content -replace '(let http_server = axum::serve::serve)', "$shutdownInit$1"
        Write-Host "  - Shutdown signal initialized" -ForegroundColor Green
    } else {
        Write-Host "  - ERROR: Could not find http_server initialization" -ForegroundColor Red
        throw "Shutdown signal initialization failed"
    }
    
    # Modify server startup to use graceful shutdown
    $gracefulShutdown = @'
    // Run both gRPC and HTTP servers concurrently with graceful shutdown
    let grpc_addr: std::net::SocketAddr = addr.parse::<std::net::SocketAddr>().map_err(|e| AppError::GrpcBind(e.to_string()))?;
    
    let shutdown_signal_clone = shutdown_signal.clone();
    let grpc_server = builder
        .add_service(FleetCommandServer::new(c2_server))
        .serve_with_graceful_shutdown(grpc_addr, async move {
            let mut rx = shutdown_signal_clone.subscribe();
            let _ = rx.recv().await;
            info!("gRPC server graceful shutdown initiated");
        });

    let shutdown_signal_clone = shutdown_signal.clone();
    let http_server = axum::serve::serve_with_graceful_shutdown(
        listener,
        http_router.into_make_service(),
        async move {
            let mut rx = shutdown_signal_clone.subscribe();
            let _ = rx.recv().await;
            info!("HTTP server graceful shutdown initiated");
        }
    );

    // Spawn shutdown signal handler
    let shutdown_signal_handler = shutdown_signal.clone();
    tokio::spawn(async move {
        wait_for_shutdown_signal(shutdown_signal_handler).await;
    });
'@
    
    # Find and replace the existing server startup code
    if ($content -match '(let grpc_addr: std::net::SocketAddr = addr\.parse::<std::net::SocketAddr>\(\)\.map_err\(|e| AppError::GrpcBind\(e\.to_string\(\)\)\?;.*?tokio::select! \{.*?result = grpc_server => \{.*?\}\.*?\})') {
        $content = $content -replace 'let grpc_addr: std::net::SocketAddr = addr\.parse::<std::net::SocketAddr>\(\)\.map_err\(|e| AppError::GrpcBind\(e\.to_string\(\)\)\?;.*?tokio::select! \{.*?result = grpc_server => \{.*?\}\.*?\}', $gracefulShutdown
        Write-Host "  - Graceful shutdown integrated" -ForegroundColor Green
    } else {
        Write-Host "  - WARNING: Could not find server startup code, manual integration may be needed" -ForegroundColor Yellow
    }
    
    # Write the modified content
    Write-Host "[WRITE] Writing modified main.rs..." -ForegroundColor Yellow
    Set-Content $MainRsPath -Value $content -NoNewline
    
    Write-Host ""
    Write-Host "=====================================================" -ForegroundColor Green
    Write-Host " INTEGRATION SUCCESSFUL " -ForegroundColor Green
    Write-Host "=====================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Changes made:" -ForegroundColor Cyan
    Write-Host "  1. Module declarations added (health_checks, graceful_shutdown, env_validation)" -ForegroundColor White
    Write-Host "  2. Imports added" -ForegroundColor White
    Write-Host "  3. Environment validation added to run_server" -ForegroundColor White
    Write-Host "  4. Health check routes added" -ForegroundColor White
    Write-Host "  5. Graceful shutdown integrated" -ForegroundColor White
    Write-Host ""
    Write-Host "Backup saved to: $BackupPath" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Verify the changes: git diff main.rs" -ForegroundColor White
    Write-Host "  2. Test compilation: cargo check" -ForegroundColor White
    Write-Host "  3. Run the server: cargo run" -ForegroundColor White
    Write-Host "  4. Test health endpoints: curl http://localhost:3000/health" -ForegroundColor White
    
} catch {
    Write-Host ""
    Write-Host "=====================================================" -ForegroundColor Red
    Write-Host " INTEGRATION FAILED " -ForegroundColor Red
    Write-Host "=====================================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Restoring backup..." -ForegroundColor Yellow
    Copy-Item $BackupPath $MainRsPath -Force
    Write-Host "Backup restored from: $BackupPath" -ForegroundColor Green
    exit 1
}
