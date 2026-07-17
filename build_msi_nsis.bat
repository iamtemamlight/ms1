@echo off
cd /d D:\ALLBRIGHTFOUR\AB4
call npm --prefix apps/dashboard run build
if errorlevel 1 (
  echo Frontend build failed.
  pause
  exit /b 1
)
cd /d D:\ALLBRIGHTFOUR\AB4\src-tauri
cargo tauri build --bundles msi,nsis
if errorlevel 1 (
  echo Tauri build failed.
  pause
  exit /b 1
)
pause
