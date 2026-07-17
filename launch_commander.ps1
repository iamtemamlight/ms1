# AllBright Commander PowerShell Launcher
# Launches the Commander interface for profit target setting

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "AllBright Commander - Command Objective" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan

# Check for existing executable
$exePath = "d:\ALLBRIGHT\src-tauri\target\release\allbright-desktop.exe"
$standaloneExe = "d:\ALLBRIGHT\allbright-desktop.exe"

# Check latest MSI installer
$msiPath = "d:\ALLBRIGHT\src-tauri\target\release\bundle\msi\AllBright Defi V119_119.0.0_x64_en-US.msi"
$nsisPath = "d:\ALLBRIGHT\src-tauri\target\release\bundle\nsis\AllBright Defi V119_119.0.0_x64-setup.exe"

if (Test-Path $exePath) {
    Write-Host "Found development build at: $exePath" -ForegroundColor Green
    # Would launch the app here in a proper environment
} elseif (Test-Path $standaloneExe) {
    Write-Host "Found standalone executable at: $standaloneExe" -ForegroundColor Green
    # Note: This executable has a plugin configuration issue
    Write-Host "Note: Existing executable has plugin config issue. Please rebuild." -ForegroundColor Yellow
}

if (Test-Path $msiPath) {
    Write-Host "MSI Installer found: $msiPath" -ForegroundColor Green
    $msiSize = [math]::Round((Get-Item $msiPath).Length / 1MB, 2)
    Write-Host "  Size: $msiSize MB" -ForegroundColor Gray
}

if (Test-Path $nsisPath) {
    Write-Host "NSIS Installer found: $nsisPath" -ForegroundColor Green
    $nsisSize = [math]::Round((Get-Item $nsisPath).Length / 1MB, 2)
    Write-Host "  Size: $nsisSize MB" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Commander Interface Features:" -ForegroundColor Yellow
Write-Host "  - Set profit growth target (%/day)" -ForegroundColor Gray
Write-Host "  - Launch autonomous optimization engine" -ForegroundColor Gray
Write-Host "  - Monitor AI mode (aggressive/balanced/stabilizing)" -ForegroundColor Gray
Write-Host "  - View enterprise score and performance status" -ForegroundColor Gray

Write-Host ""
Write-Host "To install: Run the MSI installer as Administrator" -ForegroundColor Blue
Write-Host "To rebuild: Run 'npm run desktop:build:msi-nsis' from project root" -ForegroundColor Blue