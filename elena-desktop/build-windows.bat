@echo off
chcp 65001 >nul
echo Building single Windows EXE...
echo.

set ROOT=%~dp0..
set BIN=%~dp0src-tauri\resources\bin
if not exist "%BIN%" mkdir "%BIN%"

echo [1/4] elena-core...
cd /d "%ROOT%\elena-core"
cargo build --release
if errorlevel 1 exit /b 1

echo [2/4] elena-gateway...
cd /d "%ROOT%\elena-gateway"
cargo build --release
if errorlevel 1 exit /b 1

echo [3/4] Copy bins...
copy /Y "%ROOT%\elena-core\target\release\elena-core.exe" "%BIN%\"
copy /Y "%ROOT%\elena-gateway\target\release\elena-gateway.exe" "%BIN%\"

echo [4/4] Tauri build...
cd /d "%~dp0"
call npm run tauri build
if errorlevel 1 exit /b 1

echo Done. Installer: src-tauri\target\release\bundle\nsis\*.exe
pause
