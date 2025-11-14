@echo off
REM Install script for Kandil Code on Windows
REM This script downloads and installs the latest version of Kandil Code

setlocal enabledelayedexpansion

REM Default installation directory
set "INSTALL_DIR=%LOCALAPPDATA%\kandil_code"

REM GitHub repository
set "REPO=Kandil7/kandil_code"

REM Function to detect architecture
for /f "tokens=3" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v "PROCESSOR_ARCHITECTURE" 2^>nul') do (
    set "ARCH=%%a"
)

REM Determine the appropriate binary name
if /i "%ARCH%"=="AMD64" (
    set "BINARY_NAME=kandil_code-windows-x86_64.exe"
) else if /i "%ARCH%"=="x86" (
    set "BINARY_NAME=kandil_code-windows-x86_64.exe"
) else if /i "%ARCH%"=="ARM64" (
    set "BINARY_NAME=kandil_code-windows-arm64.exe"
) else (
    echo Unsupported architecture: %ARCH%
    exit /b 1
)

echo Detected architecture: %ARCH%
echo Binary name: %BINARY_NAME%

REM Get the latest release version using PowerShell
for /f "usebackq tokens=*" %%i in (`powershell -Command "(Invoke-RestMethod -Uri 'https://api.github.com/repos/%REPO%/releases/latest').tag_name"`) do (
    set "LATEST_VERSION=%%i"
)

if "%LATEST_VERSION%"=="" (
    echo Error: Could not determine the latest version
    exit /b 1
)

REM Remove the 'v' prefix if present
set "LATEST_VERSION=%LATEST_VERSION:v=%"

echo Latest version: !LATEST_VERSION!

REM Create the download URL
set "DOWNLOAD_URL=https://github.com/%REPO%/releases/download/v!LATEST_VERSION!/!BINARY_NAME!"

echo Downloading from: !DOWNLOAD_URL!

REM Create a temporary directory
set "TEMP_DIR=%TEMP%\kandil_install_%RANDOM%"
mkdir "%TEMP_DIR%"
cd /d "%TEMP_DIR%"

REM Download the binary using PowerShell
powershell -Command "Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile 'kandil.exe'"

REM Check if download was successful
if not exist "kandil.exe" (
    echo Error: Download failed
    cd /d "%~dp0"
    rmdir "%TEMP_DIR%" /s /q
    exit /b 1
)

REM Create installation directory if it doesn't exist
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Copy the binary to the installation directory
copy "kandil.exe" "%INSTALL_DIR%\kandil.exe"

REM Add the installation directory to PATH if not already there
for /f "usebackq tokens=*" %%i in (`powershell -Command "[Environment]::GetEnvironmentVariable('Path', 'User')"`) do (
    set "USER_PATH=%%i"
)

echo !USER_PATH! | findstr /C:"!INSTALL_DIR!" >nul
if errorlevel 1 (
    echo Adding installation directory to PATH...
    powershell -Command "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';%INSTALL_DIR%', 'User')"
    echo Please restart your command prompt to use kandil.
) else (
    echo Installation directory already in PATH.
)

REM Clean up
cd /d "%~dp0"
rmdir "%TEMP_DIR%" /s /q

echo Kandil Code has been installed to %INSTALL_DIR%\kandil.exe
echo Run 'kandil --help' to get started