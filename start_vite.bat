@echo off
REM Start Vite dev server with Express simulation backend on :3002
cd /d d:\MS1\AB4\apps\dashboard
call npm install
set NODE_ENV=development
set VITE_API_BASE=http://localhost:3002
npm run dev