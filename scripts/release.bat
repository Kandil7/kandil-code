@echo off
REM Release script for Kandil Code on Windows

setlocal enabledelayedexpansion

REM Check if we're in the project root
if not exist "Cargo.toml" (
    echo Error: Must be run from project root
    exit /b 1
)

REM Check if git working directory is clean
for /f "delims=" %%i in ('git status --porcelain') do (
    if not "%%i" == "" (
        echo Error: Git working directory not clean
        git status
        exit /b 1
    )
)

REM Get the new version from command line argument
if "%~1"=="" (
    echo Usage: %0 ^<version^>
    echo Example: %0 1.0.0
    exit /b 1
)

set NEW_VERSION=%1

REM Validate version format (semantic versioning - basic check)
echo %NEW_VERSION% | findstr /r "^[0-9]*\.[0-9]*\.[0-9]*" >nul
if errorlevel 1 (
    echo Error: Invalid version format. Use semantic versioning (e.g., 1.0.0)
    exit /b 1
)

echo Preparing release for version: %NEW_VERSION%

REM Update version in Cargo.toml using PowerShell
powershell -Command "(Get-Content Cargo.toml) -replace '^version = \".*\"', \"version = `"%NEW_VERSION%`"\" | Set-Content Cargo.toml"

REM Commit the version change
echo Committing version update
git add Cargo.toml
git commit -m "Prepare for release v%NEW_VERSION%"

REM Create and push tag
echo Creating and pushing git tag v%NEW_VERSION%
git tag -a v%NEW_VERSION% -m "Release version %NEW_VERSION%"
git push origin v%NEW_VERSION%

echo Release preparation complete!
echo A GitHub release will be created automatically via GitHub Actions.
echo Check https://github.com/Kandil7/kandil_code/releases for the draft release.