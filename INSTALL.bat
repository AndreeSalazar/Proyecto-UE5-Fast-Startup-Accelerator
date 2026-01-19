@echo off
REM Fast Startup Accelerator - Installer
REM Copyright 2026 Eddi Andreé Salazar Matos - Apache 2.0

title Fast Startup Accelerator - Installer

echo.
echo  ============================================
echo   FAST STARTUP ACCELERATOR - INSTALLER
echo  ============================================
echo.

set UE5_PATH=C:\Program Files\Epic Games\UE_5.7
set PLUGIN_DEST=%UE5_PATH%\Engine\Plugins\Editor\FastStartup
set SOURCE_PATH=%~dp0ue5-plugin

echo Checking UE5 installation...

if not exist "%UE5_PATH%" (
    echo [ERROR] UE5 not found at: %UE5_PATH%
    echo Please modify this script with your UE5 installation path.
    pause
    exit /b 1
)

echo [OK] UE5 found at: %UE5_PATH%
echo.

echo Installing plugin to: %PLUGIN_DEST%
echo.

if exist "%PLUGIN_DEST%" (
    echo Removing old installation...
    rmdir /s /q "%PLUGIN_DEST%"
)

echo Copying plugin files...
xcopy /s /e /i /y "%SOURCE_PATH%\*" "%PLUGIN_DEST%\"

echo.
echo Copying CLI executable...
copy /y "%~dp0rust-analyzer\target\release\ue5-fast-startup.exe" "%PLUGIN_DEST%\Binaries\"

echo.
echo  ============================================
echo   INSTALLATION COMPLETE!
echo  ============================================
echo.
echo Plugin installed to:
echo   %PLUGIN_DEST%
echo.
echo To use:
echo   1. Open any UE5 project
echo   2. Run: %PLUGIN_DEST%\Scripts\BuildProjectCache.bat
echo   3. Pass your project path as argument
echo.
echo Example:
echo   BuildProjectCache.bat "C:\MyProjects\MyGame"
echo.

pause
