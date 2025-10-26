@echo off
title Nexus GitHub Orchestrator

cd /d "%~dp0"

echo ========================================
echo   NEXUS GITHUB ORCHESTRATOR v2.0
echo   Multi-Account GitHub Actions Runner
echo ========================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Rust/Cargo not found!
    echo.
    echo Install from: https://rustup.rs/
    echo.
    pause
    exit /b 1
)

REM Check if config exists
if not exist "config\tokens.txt" (
    echo WARNING: config\tokens.txt not found!
    echo.
    echo Run: scripts\setup.sh first
    echo.
    pause
    exit /b 1
)

echo Starting orchestrator...
echo.

cargo run --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ERROR: Orchestrator exited with error code %ERRORLEVEL%
    pause
)
