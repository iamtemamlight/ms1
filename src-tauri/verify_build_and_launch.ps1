Set-Location $PSScriptRoot
$ErrorActionPreference = 'Stop'

Write-Host "=== AllBright Desktop 2026/V119 Build Verification ===" -ForegroundColor Cyan

# Check bundle output
$bundles = Get-ChildItem -Path 'target\release\bundle' -Recurse -ErrorAction SilentlyContinue
if ($bundles) {
    Write-Host "Bundle files found:" -ForegroundColor Green
    foreach ($b in $bundles) { Write-Host "  " $b.FullName }
} else {
    Write-Host "No bundle files found. Build may be required." -ForegroundColor Yellow
}

# Check main executable
$exe = Get-Item -Path 'target\release\allbright-desktop.exe' -ErrorAction SilentlyContinue
if ($null -ne $exe) {
    Write-Host "`nExecutable found: $($exe.FullName)" -ForegroundColor Green
    Write-Host "File size: $([math]::Round($exe.Length / 1MB, 2)) MB" -ForegroundColor Green
    
    # Verify executable can launch
    $p = Start-Process -FilePath $exe.FullName -WindowStyle Hidden -PassThru
    Start-Sleep -Seconds 3
    
    if (-not $p.HasExited) {
        Write-Host "Process started successfully, PID=$($p.Id)" -ForegroundColor Green
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        Write-Host "Process terminated for verification." -ForegroundColor Green
    } else {
        Write-Host "Process exited with code $($p.ExitCode)" -ForegroundColor Red
    }
} else {
    Write-Host "`nExecutable not found. Run 'npm run tauri build' to build." -ForegroundColor Red
}

Write-Host "`nVerification complete."