@echo off
REM AllBright Defi Software Engineering Ltd. 2026/V119 - Verification Script
echo =============================================
echo AllBright Defi Software Engineering Ltd. 2026/V119
echo Verification and Installation Script
echo =============================================
echo.

REM Check for existing installers
echo Checking for existing installers...
echo.

if exist "d:\ALLBRIGHT\AllBright_Desktop_V91.msi" (
    echo Found: AllBright_Desktop_V91.msi
)
if exist "d:\ALLBRIGHT\AllBright_Desktop_V91_Setup.exe" (
    echo Found: AllBright_Desktop_V91_Setup.exe
)
if exist "d:\ALLBRIGHT\allbright-desktop.exe" (
    echo Found: allbright-desktop.exe ^(ready to launch^)
)
if exist "d:\ALLBRIGHT\src-tauri\target\release\bundle\msi\*.msi" (
    echo Found: V119 MSI installer
)
if exist "d:\ALLBRIGHT\src-tauri\target\release\bundle\nsis\*.exe" (
    echo Found: V119 NSIS installer
)

echo.
echo =============================================
echo Installation Options:
echo =============================================
echo 1. Launch existing desktop application
echo 2. Install from MSI
echo 3. Install from NSIS EXE
echo 4. Exit
echo.

choice /C 1234 /M "Select option"
if errorlevel 4 exit /b 0
if errorlevel 3 goto nsis_install
if errorlevel 2 goto msi_install
if errorlevel 1 goto launch_app

:launch_app
echo Launching AllBright Desktop...
if exist "d:\ALLBRIGHT\allbright-desktop.exe" (
    start "" "d:\ALLBRIGHT\allbright-desktop.exe"
) else (
    echo No executable found. Please build first.
)
goto end

:msi_install
echo Run: msiexec /i "AllBright_Desktop_V91.msi"
goto end

:nsis_install
echo Run the NSIS installer manually.
goto end

:end
echo.
echo Verification complete.
echo Application: AllBright Defi Software Engineering Ltd. 2026/V119

# Step 4: List output files
Write-Host "[4/5] Build outputs:" -ForegroundColor Yellow
Write-Host "MSI Installers:" -ForegroundColor Cyan
Get-ChildItem "target\release\bundle\msi" -Recurse -Include *.msi 2>$null | ForEach-Object {
    Write-Host "  $($_.FullName)" -ForegroundColor White
}
Write-Host ""
Write-Host "NSIS Installers:" -ForegroundColor Cyan
Get-ChildItem "target\release\bundle\nsis" -Recurse -Include *.exe 2>$null | ForEach-Object {
    Write-Host "  $($_.FullName)" -ForegroundColor White
}
Write-Host ""

# Step 5: Launch the application
Write-Host "[5/5] Launching AllBright Defi V119..." -ForegroundColor Yellow
Set-Location "d:\ALLBRIGHT"

$exePath = $null
$possiblePaths = @(
    "src-tauri\target\release\allbright-defi-v119.exe",
    "src-tauri\target\release\bundle\nsis\allbright-defi-v119.exe",
    "allbright-desktop.exe"
)

foreach ($path in $possiblePaths) {
    if (Test-Path $path) {
        $exePath = $path
        break
    }
}

if ($exePath) {
    Write-Host "Launching from: $exePath" -ForegroundColor Green
    Start-Process -FilePath $exePath -WorkingDirectory "d:\ALLBRIGHT"
    Write-Host "Application launched successfully!" -ForegroundColor Green
} else {
    Write-Host "No executable found. Install from MSI/NSIS bundle." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=============================================" -ForegroundColor Green
Write-Host "BUILD and LAUNCH COMPLETE!" -ForegroundColor Green
Write-Host "Application: AllBright Defi Software Engineering Ltd. 2026/V119" -ForegroundColor White
Write-Host "Version: 119.0.0" -ForegroundColor White
Write-Host "=============================================" -ForegroundColor Green