@echo off
REM AllBright Commander Launcher
REM Launches the Commander interface (Command Objective page) for the AllBright Desktop application

echo ============================================
echo AllBright Commander - Command Objective
echo ============================================

REM Check if the desktop executable exists
if exist "d:\ALLBRIGHT\src-tauri\target\release\allbright-desktop.exe" (
    echo Launching AllBright Desktop (development build)...
    start "" "d:\ALLBRIGHT\src-tauri\target\release\allbright-desktop.exe"
) else if exist "d:\ALLBRIGHT\allbright-desktop.exe" (
    echo Launching AllBright Desktop (standalone)...
    start "" "d:\ALLBRIGHT\allbright-desktop.exe"
) else if exist "d:\ALLBRIGHT\src-tauri\target\release\bundle\msi\*.msi" (
    echo MSI installer found but app not installed.
    echo Please install the MSI first or run: npm run desktop:build:msi-nsis
) else (
    echo AllBright Desktop executable not found.
    echo Building frontend...
    npm run --prefix apps/dashboard build
    if errorlevel 1 (
        echo Build failed.
        pause
        exit /b 1
    )
    echo.
    echo To build full installers, run: npm run desktop:build:msi-nsis
    echo Note: Requires Windows SDK (rc.exe) for MSI/NSIS bundles.
)

echo.
echo Navigate to 'Commander' tab in the app to set profit targets.
echo The Commander interface allows you to set the profit growth target (%/day)
echo for the autonomous optimization engine.

REM Open the dashboard in browser as fallback
echo.
echo Opening Commander in browser (fallback)...
start "" "http://localhost:3000"

pause