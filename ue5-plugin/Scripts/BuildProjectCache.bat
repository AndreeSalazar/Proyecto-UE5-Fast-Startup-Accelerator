@echo off
REM Fast Startup Accelerator - Project Cache Builder
REM Copyright 2026 Eddi Andreé Salazar Matos - Apache 2.0

title Fast Startup Accelerator

echo.
echo  ============================================
echo   FAST STARTUP ACCELERATOR
echo   Rust + SIMD powered asset pipeline
echo  ============================================
echo.

set CLI_PATH=%~dp0..\Binaries\ue5-fast-startup.exe

if not exist "%CLI_PATH%" (
    echo [ERROR] CLI not found at: %CLI_PATH%
    pause
    exit /b 1
)

echo [OK] CLI found: %CLI_PATH%
echo.

if "%1"=="" (
    echo Usage: 
    echo   BuildProjectCache.bat "C:\Path\To\Your\UE5Project"
    echo.
    echo Example:
    echo   BuildProjectCache.bat "C:\Users\MyUser\Documents\Unreal Projects\MyGame"
    echo.
    set /p PROJECT_PATH="Enter project path (or drag folder here): "
) else (
    set PROJECT_PATH=%~1
)

if not exist "%PROJECT_PATH%" (
    echo [ERROR] Project path not found: %PROJECT_PATH%
    pause
    exit /b 1
)

set CACHE_PATH=%PROJECT_PATH%\Saved\FastStartup

if not exist "%CACHE_PATH%" mkdir "%CACHE_PATH%"

echo.
echo [1/3] Scanning project assets...
echo      Project: %PROJECT_PATH%
echo.

"%CLI_PATH%" scan --project "%PROJECT_PATH%" -v

echo.
echo [2/3] Analyzing dependencies...
echo.

"%CLI_PATH%" analyze --project "%PROJECT_PATH%" --output "%CACHE_PATH%\analysis.json" -v

echo.
echo [3/3] Building startup cache...
echo.

"%CLI_PATH%" cache --project "%PROJECT_PATH%" --output "%CACHE_PATH%\startup.uefast" -v

echo.
echo  ============================================
echo   CACHE BUILD COMPLETE!
echo  ============================================
echo.
echo  Cache location: %CACHE_PATH%\startup.uefast
echo  Analysis:       %CACHE_PATH%\analysis.json
echo.
echo  Your UE5 project will now load faster!
echo.

pause
