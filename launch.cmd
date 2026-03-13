@echo off
REM Chetna Launcher Script for Windows
REM Usage: launch.cmd [command]
REM Commands:
REM   (none)      Start Chetna (kills existing if running)
REM   --api       Start and wait for server (for AI agents)
REM   --stop      Stop running Chetna
REM   --check     Check if Chetna is running

setlocal enabledelayedexpansion

set PORT=%CHETNA_PORT%=1987
set DB_PATH=%CHETNA_DB_PATH%=.\ChetnaData\chetna.db
set HOST=%CHETNA_HOST%=0.0.0.0

set "PID_FILE=chetna.pid"

REM Find process by port using netstat
find_process() {
    for /f "tokens=5" %%a in ('netstat -ano ^| findstr :%PORT% ^| findstr LISTENING') do (
        set "PID=%%a"
        goto :found
    )
    set "PID="
    :found
}

REM Kill existing process
:kill_existing
if "%~1"=="--no-kill" goto :skip_kill

call :find_process
if defined PID (
    echo Found existing Chetna on port %PORT% (PID: %PID%)
    echo Killing process...
    taskkill /F /PID %PID% >nul 2>&1
    timeout /t 2 /nobreak >nul
    echo Process killed
)
:skip_kill
goto :eof

REM Check if running
:check_running
call :find_process
if defined PID (
    echo Chetna is running on port %PORT% (PID: %PID%)
    exit /b 0
) else (
    echo Chetna is not running
    exit /b 1
)

REM Stop Chetna
:stop_chetna
call :find_process
if defined PID (
    echo Stopping Chetna (PID: %PID%)...
    taskkill /F /PID %PID% >nul 2>&1
    timeout /t 2 /nobreak >nul
    echo Chetna stopped
) else (
    echo Chetna is not running
)
goto :eof

REM Start Chetna
:start_chetna
echo Starting Chetna...
echo   Host: %HOST%
echo   Port: %PORT%
echo   Database: %DB_PATH%

REM Ensure data directory exists
for %%i in (%DB_PATH%) do set "DB_DIR=%%~dpi"
if not exist "%DB_DIR%" mkdir "%DB_DIR%"

REM Save PID
echo %PID% > %PID_FILE%

REM Start cargo run in background
start "Chetna" cmd /c "cargo run"

echo Chetna started
echo.
echo Dashboard: http://localhost:%PORT%
echo API:       http://localhost:%PORT%/api
echo.
goto :eof

REM API mode - for AI agents
:api_start
call :check_running
if !errorlevel! equ 0 (
    echo {"status":"running","port":%PORT%,"message":"Chetna already running"}
    exit /b 0
)

call :kill_existing
call :start_chetna

echo Waiting for server to start...
for /L %%i in (1,1,30) do (
    curl -s http://localhost:%PORT%/health >nul 2>&1
    if !errorlevel! equ 0 (
        echo {"status":"started","port":%PORT%,"message":"Chetna started successfully"}
        exit /b 0
    )
    timeout /t 1 /nobreak >nul
)
echo {"status":"error","message":"Timeout waiting for Chetna to start"}
exit /b 1

REM Main
if "%~1"=="--api" goto :api_start
if "%~1"=="--stop" goto :stop_chetna
if "%~1"=="--check" goto :check_running
if "%~1"=="--help" goto :help
if "%~1"=="-h" goto :help

call :kill_existing %~1
call :start_chetna
goto :eof

:help
echo Chetna Launcher
echo.
echo Usage: launch.cmd [command]
echo.
echo Commands:
echo   (none)      Start Chetna (kills existing if running)
echo   --api       Start and wait for server (for AI agents)
echo   --stop      Stop running Chetna
echo   --check     Check if Chetna is running
echo   --help      Show this help
goto :eof
