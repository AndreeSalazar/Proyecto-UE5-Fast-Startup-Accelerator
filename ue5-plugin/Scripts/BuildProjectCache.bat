@echo off
REM Fast Startup Accelerator - ULTRA OPTIMIZED
REM Copyright 2026 Eddi Andreé Salazar Matos - Apache 2.0

title Fast Startup Accelerator - ULTRA

echo.
echo  ============================================
echo   FAST STARTUP ACCELERATOR - ULTRA
echo   Rust + SIMD + NASM powered pipeline
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
    echo   BuildProjectCache.bat "C:\Path\To\Your\UE5Project" [mode]
    echo.
    echo Modes:
    echo   turbo  - Ultra-fast with sampling (default)
    echo   full   - Complete analysis and cache
    echo.
    echo Example:
    echo   BuildProjectCache.bat "C:\MyProjects\MyGame" turbo
    echo.
    set /p PROJECT_PATH="Enter project path: "
    set MODE=turbo
) else (
    set PROJECT_PATH=%~1
    if "%2"=="" (set MODE=turbo) else (set MODE=%2)
)

if not exist "%PROJECT_PATH%" (
    echo [ERROR] Project path not found: %PROJECT_PATH%
    pause
    exit /b 1
)

set CACHE_PATH=%PROJECT_PATH%\Saved\FastStartup

if not exist "%CACHE_PATH%" mkdir "%CACHE_PATH%"

echo.
echo  Mode: %MODE%
echo  Project: %PROJECT_PATH%
echo.

if /i "%MODE%"=="turbo" goto :turbo_mode
if /i "%MODE%"=="full" goto :full_mode
goto :turbo_mode

:turbo_mode
echo  ============================================
echo   TURBO MODE - Maximum Speed
echo  ============================================
echo.

"%CLI_PATH%" turbo --project "%PROJECT_PATH%" --output "%CACHE_PATH%\startup.uefast"

goto :done

:full_mode
echo  ============================================
echo   FULL MODE - Complete Analysis
echo  ============================================
echo.

echo [1/3] Scanning project assets...
"%CLI_PATH%" scan --project "%PROJECT_PATH%" -v

echo.
echo [2/3] Analyzing dependencies...
"%CLI_PATH%" analyze --project "%PROJECT_PATH%" --output "%CACHE_PATH%\analysis.json" -v

echo.
echo [3/3] Building startup cache...
"%CLI_PATH%" cache --project "%PROJECT_PATH%" --output "%CACHE_PATH%\startup.uefast" -v

goto :done

:done
echo.
echo  ============================================
echo   CACHE BUILD COMPLETE!
echo  ============================================
echo.
echo  Cache: %CACHE_PATH%\startup.uefast
echo.
echo  To verify changes quickly:
echo    %CLI_PATH% quick-verify -c "%CACHE_PATH%\startup.uefast" -p "%PROJECT_PATH%"
echo.

pause
