# Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ProjectRoot = 'D:\ALLBRIGHT'
$TauriDir   = Join-Path $ProjectRoot 'src-tauri'

Write-Host "`n[1/2] Building Tauri release + bundle..."
Push-Location $TauriDir

# Determine if WiX Toolset is available
$hasWix = $false
$wixPaths = @(
    "C:\Program Files\WiX Toolset v3.14\bin\candle.exe",
    "C:\Program Files (x86)\WiX Toolset v3.11\bin\candle.exe",
    "C:\Program Files\WiX Toolset v3.11\bin\candle.exe"
)
foreach ($path in $wixPaths) {
    if (Test-Path $path) { $hasWix = $true; break }
}
if (Get-Command candle.exe -ErrorAction SilentlyContinue) {
    $hasWix = $true
}

# Select build bundle target based on WiX presence
$bundleTarget = "nsis"
if ($hasWix) {
    Write-Host "WiX Toolset detected. Will attempt MSI bundling." -ForegroundColor Green
    $bundleTarget = "msi"
} else {
    Write-Host "WiX Toolset NOT detected. Using NSIS bundling fallback (automatic tool setup)." -ForegroundColor Yellow
}

# Run build using local Tauri CLI (v1) via npx
Write-Host "Executing build command: npx tauri build --bundles $bundleTarget" -ForegroundColor Cyan
npx tauri build --bundles $bundleTarget

Pop-Location

# Locate generated bundle
$MsiBundleDir = Join-Path $TauriDir 'target\release\bundle\msi'
$NsisBundleDir = Join-Path $TauriDir 'target\release\bundle\nsis'

$installerPath = $null
$isMsi = $false

if ($bundleTarget -eq "msi" -and (Test-Path $MsiBundleDir)) {
    $installer = Get-ChildItem $MsiBundleDir -Filter '*.msi' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($installer) {
        $installerPath = $installer.FullName
        $isMsi = $true
    }
}

if (-not $installerPath -and (Test-Path $NsisBundleDir)) {
    $installer = Get-ChildItem $NsisBundleDir -Filter '*.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($installer) {
        $installerPath = $installer.FullName
        $isMsi = $false
    }
}

if (-not $installerPath) {
    Write-Host "`nNo installer package (.msi or .exe) found in release bundles." -ForegroundColor Red
    exit 1
}

Write-Host "`n[2/2] Installing: $installerPath"

if ($isMsi) {
    Write-Host "Running MSI silent installation..." -ForegroundColor Yellow
    $args = @('/i', $installerPath, '/qn', '/norestart')
    Start-Process msiexec.exe -ArgumentList $args -Wait -NoNewWindow
} else {
    Write-Host "Running NSIS silent installation..." -ForegroundColor Yellow
    # NSIS installer accepts /S for silent install
    $args = @('/S')
    Start-Process $installerPath -ArgumentList $args -Wait -NoNewWindow
}

Write-Host "`nInstaller finished. App should be in:" -ForegroundColor Green
Write-Host "$env:LOCALAPPDATA\Allbright 059\" -ForegroundColor Cyan