@echo off
echo ========================================
echo ALLBRIGHT Tauri Desktop Build Script
echo ========================================
echo.

echo [1/4] Checking prerequisites...
:: Check if npm is available
where npm >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: npm not found. Please install Node.js.
    exit /b 1
)

:: Check if cargo is available
where cargo >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: cargo not found. Please install Rust.
    exit /b 1
)

echo [2/4] Building frontend dashboard...
cd /d d:\ALLBRIGHT\apps\dashboard
call npm install
call npm run build

if %ERRORLEVEL% neq 0 (
    echo ERROR: Frontend build failed!
    exit /b 1
)
echo Frontend build SUCCESS

echo [3/4] Building Tauri desktop app...
cd /d d:\ALLBRIGHT\src-tauri
cargo tauri build --bundles msi,nsis

if %ERRORLEVEL% neq 0 (
    echo ERROR: Tauri build failed!
    exit /b 1
)

echo [4/4] Build complete!
echo.
echo MSI installer: src-tauri\target\release\bundle\msi\
echo NSIS installer: src-tauri\target\release\bundle\nsis\
echo.
echo SUCCESS - Desktop installation builds completed!
pause
