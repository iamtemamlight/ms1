@echo off
cd /d d:\MS1\AB4\backend
set HTTP_BIND_ADDR=0.0.0.0:3001
start "" "%~dp0backend\target\release\allbright-c2-backend.exe"
