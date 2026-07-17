@echo off
REM Build the dashboard assets from the src-tauri directory.
cd /d %~dp0
cd ..\apps\dashboard
npm run build
