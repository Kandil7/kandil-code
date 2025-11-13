#!/bin/bash
# Install script for Kandil Code
# This script downloads and installs the latest version of Kandil Code

set -e

# Default installation directory
DEFAULT_INSTALL_DIR="/usr/local/bin"
INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

# GitHub repository
REPO="Kandil7/kandil_code"

# Function to detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case $ARCH in
        x86_64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        armv*)
            ARCH="arm"
            ;;
    esac

    # Determine the appropriate binary name
    case $OS in
        linux*)
            BINARY_NAME="kandil_code-linux-$ARCH"
            ;;
        darwin*)
            BINARY_NAME="kandil_code-macos-$ARCH"
            ;;
        *)
            echo "Unsupported platform: $OS $ARCH"
            exit 1
            ;;
    esac
}

# Function to get the latest release version
get_latest_version() {
    # Use GitHub API to get the latest release
    LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$LATEST_VERSION" ]; then
        echo "Error: Could not determine the latest version"
        exit 1
    fi
    # Remove the 'v' prefix if present
    LATEST_VERSION=${LATEST_VERSION#v}
}

# Function to download and install
download_and_install() {
    echo "Detecting platform..."
    detect_platform
    
    echo "Getting latest version..."
    get_latest_version
    
    echo "Latest version: $LATEST_VERSION"
    echo "Platform: $OS $ARCH"
    echo "Binary name: $BINARY_NAME"
    
    # Construct the download URL
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/v$LATEST_VERSION/$BINARY_NAME"
    
    echo "Downloading from: $DOWNLOAD_URL"
    
    # Create a temporary directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Download the binary
    if command -v curl >/dev/null 2>&1; then
        curl -L -o kandil_code "$DOWNLOAD_URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -O kandil_code "$DOWNLOAD_URL"
    else
        echo "Error: Neither curl nor wget is available"
        exit 1
    fi
    
    # Make the binary executable
    chmod +x kandil_code
    
    # Create installation directory if it doesn't exist
    sudo mkdir -p "$INSTALL_DIR"
    
    # Install the binary
    sudo cp kandil_code "$INSTALL_DIR/kandil"
    
    # Clean up
    cd - > /dev/null
    rm -rf "$TEMP_DIR"
    
    echo "Kandil Code has been installed to $INSTALL_DIR/kandil"
    echo "Run 'kandil --help' to get started"
}

# Run the installation
download_and_install