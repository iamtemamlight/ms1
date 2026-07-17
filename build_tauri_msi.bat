@echo off
echo =============================================
echo AllBright Defi Software Engineering Ltd. 2026/V119 - MSI Build
echo =============================================
echo.
cd /d D:\ALLBRIGHTFOUR\AB4\src-tauri
cargo tauri build --bundles msi --ci
if errorlevel 1 (
    echo Build FAILED!
    pause
    exit /b 1
)
echo.
echo Build complete! Check src-tauri/target/release/bundle/msi for output files.
dir /s /b "D:\ALLBRIGHTFOUR\AB4\src-tauri\target\release\bundle\msi\*.msi" 2>nul
pause
