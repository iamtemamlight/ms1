@echo off
REM AllBright Tauri Desktop Deployment Script
REM Builds MSI and NSIS installers for Windows

echo ============================================
echo AllBright Desktop - Tauri Installer Build
echo ============================================

cd /d "%~dp0"
cd ..\src-tauri

echo [1/4] Installing Tauri CLI if needed...
cargo install tauri-cli --locked 2>nul

echo [2/4] Building dashboard frontend...
cd ..\apps\dashboard
call npm run build

echo [3/4] Building Tauri application...
cd ..\..\src-tauri
cargo tauri build --release

echo [4/4] Verifying installers...
cd target\release\bundle
if exist msi\allbright-desktop_*.msi (
    echo MSI Installer: Found
    dir msi\*.msi
) else (
    echo MSI Installer: Not found
)
if exist nsis\AllBright-Desktop_*.exe (
    echo NSIS Installer: Found
    dir nsis\*.exe
) else (
    echo NSIS Installer: Not found
)

echo ============================================
echo Build Complete - Check target\release\bundle
echo ============================================