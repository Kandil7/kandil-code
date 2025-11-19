# PowerShell script for Kandil Code release artifacts
# This script builds all necessary artifacts for release

Write-Output "Starting Kandil Code release build..."

# Create release directory
New-Item -ItemType Directory -Path "release_artifacts" -Force
Set-Location "release_artifacts"

# Define supported targets
$DesktopTargets = @(
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu", 
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc"
)

$MobileTargets = @(
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "x86_64-linux-android",
    "i686-linux-android"
)

# Function to build desktop targets
function Build-Desktop {
    param([string]$Target)
    
    Write-Output "Building for desktop target: $Target"
    
    # Add the target if not already available
    rustup target add $Target
    
    # Build for the target
    cargo build --release --target $Target
    
    # Package the binary
    $BinaryName = "kandil"
    if ($Target -like "*windows*") {
        $BinaryName = "kandil.exe"
    }
    
    $OutputFile = "kandil-$Target"
    $BinaryPath = "target\$Target\release\$BinaryName"
    
    if ($Target -like "*windows*") {
        # On Windows, create zip
        Compress-Archive -LiteralPath $BinaryPath -DestinationPath "$OutputFile.zip"
        (Get-FileHash -Algorithm SHA256 "$OutputFile.zip").Hash.ToLower() | Out-File "$OutputFile.sha256" -NoNewline
    } else {
        # On Unix-like systems, we'd create tar.gz (not supported natively on Windows)
        # For now, we'll just create the checksum
        if (Test-Path $BinaryPath) {
            (Get-FileHash -Algorithm SHA256 $BinaryPath).Hash.ToLower() | Out-File "$OutputFile.sha256" -NoNewline
        }
    }
}

# Build desktop targets
Write-Output "Building desktop targets..."
foreach ($Target in $DesktopTargets) {
    Build-Desktop -Target $Target
}

# Create a summary of all artifacts
Write-Output "Build Summary:"
Get-ChildItem -Path . -Recurse -Include "*.zip", "*.sha256"

Write-Output "Release build completed! Artifacts are in release_artifacts/"