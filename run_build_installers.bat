@echo off
echo =============================================
echo AllBright Defi Software Engineering Ltd. 2026/V119 - Build Process Starting
echo =============================================
echo.
echo [1/4] Building frontend...
cd d:\ALLBRIGHT\apps\dashboard
call node_modules\.bin\vite build
if errorlevel 1 (
    echo Frontend build FAILED!
    pause
    exit /b 1
)
echo Frontend build SUCCESS!
echo.
echo [2/4] Verifying dist folder...
if not exist "dist\index.html" (
    echo dist/index.html NOT FOUND!
    pause
    exit /b 1
)
echo Dist folder verified!
echo.
echo [3/4] Building Tauri applications (MSI + NSIS)...
cd d:\ALLBRIGHT\src-tauri
call cargo tauri build --bundles msi,nsis
if errorlevel 1 (
    echo Tauri build FAILED!
    pause
    exit /b 1
)
echo Tauri build SUCCESS!
echo.
echo [4/4] Build outputs:
dir /s /b "d:\ALLBRIGHT\src-tauri\target\release\bundle\*.msi" 2>nul
dir /s /b "d:\ALLBRIGHT\src-tauri\target\release\bundle\*.exe" 2>nul
echo.
echo =============================================
echo BUILD COMPLETE!
echo =============================================
pause
