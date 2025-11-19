#!/bin/bash
# Build script for Kandil Code release artifacts
# This script builds all necessary artifacts for release

set -e  # Exit on any error

echo "Starting Kandil Code release build..."

# Create release directory
mkdir -p release_artifacts
cd release_artifacts

# Define supported targets
DESKTOP_TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu" 
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
    "aarch64-pc-windows-msvc"
)

MOBILE_TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
    "i686-linux-android"
)

# Function to build desktop targets
build_desktop() {
    local target=$1
    echo "Building for desktop target: $target"
    
    # Add the target if not already available
    rustup target add $target
    
    # Build for the target
    cargo build --release --target $target
    
    # Package the binary
    local binary_name="kandil"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="kandil.exe"
    fi
    
    local output_file="kandil-$target"
    local binary_path="target/$target/release/$binary_name"
    
    if [[ "$target" == *"windows"* ]]; then
        # On Windows, create zip
        zip "$output_file.zip" "$binary_path"
        sha256sum "$output_file.zip" > "$output_file.sha256"
    else
        # On Unix-like systems, create tar.gz
        tar czf "$output_file.tar.gz" -C "target/$target/release" "$binary_name"
        if command -v sha256sum &> /dev/null; then
            sha256sum "$output_file.tar.gz" > "$output_file.sha256"
        else
            shasum -a 256 "$output_file.tar.gz" > "$output_file.sha256"
        fi
    fi
}

# Function to build mobile targets
build_mobile() {
    local target=$1
    echo "Building for mobile target: $target"
    
    # Add the target
    rustup target add $target
    
    # Install required tools for Android if needed
    if [[ "$target" == *"android"* ]]; then
        echo "Setting up Android build environment..."
        # This is a simplified Android setup
        # In a real scenario, you'd need Android NDK setup
        rustup target add $target
    fi
    
    # Build as a library for mobile
    cargo build --release --target $target --lib
    
    # Package mobile library
    local lib_name="libkandil.so"
    if [[ "$target" == *"windows"* ]]; then
        lib_name="kandil.dll"
    elif [[ "$target" == *"darwin"* ]]; then
        lib_name="libkandil.dylib"
    fi
    
    local mobile_package="mobile-kandil-$target"
    mkdir -p "$mobile_package"
    cp target/$target/release/$lib_name "$mobile_package/" 2>/dev/null || echo "No library found for $target, creating placeholder"
    
    # Create checksums
    find target/$target/release/ -name "*kandil*" -type f -exec sha256sum {} \; > "$mobile_package/checksums.txt" || echo "No checksums for $target"
    
    # Create archive
    tar czf "mobile-$target.tar.gz" -C . "$mobile_package"
}

# Build desktop targets
echo "Building desktop targets..."
for target in "${DESKTOP_TARGETS[@]}"; do
    build_desktop "$target"
done

# Build mobile targets (if environment supports it)
echo "Building mobile targets..."
for target in "${MOBILE_TARGETS[@]}"; do
    build_mobile "$target"
done

echo "Release build completed! Artifacts are in release_artifacts/"
ls -la release_artifacts/

# Create a summary of all artifacts
echo "Build Summary:"
find release_artifacts/ -type f -name "*.tar.gz" -o -name "*.zip" -o -name "*.sha256" | sort