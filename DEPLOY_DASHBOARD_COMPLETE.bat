@echo off
REM ============================================================================
REM AllBright V119 - Complete Dashboard Deployment Script
REM ============================================================================
REM This script deploys the dashboard on localhost per PORT_MAPPING.md
REM Prerequisites: Backend Rust server running on port 3000
REM ============================================================================

echo.
echo ================================================================================
echo   AllBright V119 - Dashboard Deployment
echo ================================================================================
echo.

REM Check if backend is running on port 3000
echo [1/5] Checking backend service...
netstat -tulpn | findstr :3000 >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] Backend not detected on port 3000
    echo [INFO] Starting backend with docker-compose...
    cd /d d:\MS1\AB4
    docker compose up -d backend postgres redis localport-rpc
    timeout /t 15 /nobreak >nul
) else (
    echo [OK] Backend already running on port 3000
)

REM Verify backend health
echo.
echo [2/5] Verifying backend health...
curl -f http://localhost:3000/healthz >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Backend health check passed
) else (
    echo [ERROR] Backend not responding on port 3000
    echo [INFO] Please start backend manually: cargo run --bin allbright-c2-backend
    pause
    exit /b 1
)

REM Verify frontend build exists
echo.
echo [3/5] Verifying frontend build...
if not exist "apps\dashboard\dist\index.html" (
    echo [INFO] Building frontend...
    cd apps\dashboard
    npm install
    npm run build
    cd ..\..
) else (
    echo [OK] Frontend build exists
)

REM Start nginx or simple HTTP server
echo.
echo [4/5] Starting dashboard server on port 5200...
echo [INFO] Starting Node.js server to serve built frontend...

REM Kill any existing process on port 5200
netstat -tulpn | findstr :5200 >nul 2>&1
if %errorlevel% equ 0 (
    echo [WARNING] Port 5200 in use, attempting to stop existing server...
    for /f "tokens=5" %%a in ('netstat -tulpn ^| findstr :5200') do (
        taskkill /F /PID %%a >nul 2>&1
    )
    timeout /t 2 /nobreak >nul
)

REM Start Express server (serves dist/ folder on port 5200)
echo [INFO] Starting Express server on port 5200...
cd apps\dashboard
start "AllBright Dashboard" cmd /c "node server.js --port 5202"
cd ..\..

REM Wait for server to start
timeout /t 3 /nobreak >nul

REM Verify dashboard is accessible
echo.
echo [5/5] Verifying dashboard deployment...
curl -f http://localhost:5200/ >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] Dashboard is accessible at http://localhost:5200
) else (
    echo [INFO] Dashboard server starting (may take a few more seconds)...
    timeout /t 5 /nobreak >nul
    curl -f http://localhost:5200/ >nul 2>&1
    if %errorlevel% equ 0 (
        echo [OK] Dashboard is accessible at http://localhost:5200
    ) else (
        echo [ERROR] Dashboard failed to start on port 5200
        echo [INFO] Check logs in apps\dashboard\server.js
        pause
        exit /b 1
    )
)

REM Display deployment summary
echo.
echo ================================================================================
echo   DEPLOYMENT COMPLETE
echo ================================================================================
echo.
echo Services Running:
echo   - Backend HTTP:     http://localhost:3000
echo   - Backend gRPC:     localhost:50051
echo   - Backend WS:       ws://localhost:50052
echo   - Dashboard:        http://localhost:5200
echo   - PostgreSQL:       localhost:5432
echo   - Redis:            localhost:6379
echo   - LocalPort RPC:    http://localhost:8545
echo.
echo Verification Steps:
echo   1. Open browser: http://localhost:5200
echo   2. Check metrics load: http://localhost:5200/api/metrics
echo   3. Verify WebSocket: ws://localhost:5200/ws/
echo   4. Test opportunities: http://localhost:5200/api/opportunities
echo   5. Check compliance cards: http://localhost:5200/api/audit/reflections
echo.
echo Next Steps:
echo   - Run simulation tests: See DASHBOARD_DEPLOYMENT_IMPLEMENTATION_PLAN.md Phase 2
echo   - Verify compliance: See Phase 4.3 in implementation plan
echo   - Monitor logs: docker logs allbright-backend -f
echo.
echo ================================================================================
echo.

REM Keep window open
pause