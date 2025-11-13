#!/usr/bin/env bash
# Release script for Kandil Code

set -e

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must be run from project root"
    exit 1
fi

# Check if git working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Git working directory not clean"
    git status
    exit 1
fi

# Get the new version from command line argument
if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

NEW_VERSION="$1"

# Validate version format (semantic versioning)
if [[ ! "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$ ]]; then
    echo "Error: Invalid version format. Use semantic versioning (e.g., 1.0.0)"
    exit 1
fi

echo "Preparing release for version: $NEW_VERSION"

# Update version in Cargo.toml
echo "Updating Cargo.toml version to $NEW_VERSION"
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Commit the version change
echo "Committing version update"
git add Cargo.toml
git commit -m "Prepare for release v$NEW_VERSION"

# Create and push tag
echo "Creating and pushing git tag v$NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"
git push origin "v$NEW_VERSION"

echo "Release preparation complete!"
echo "A GitHub release will be created automatically via GitHub Actions."
echo "Check https://github.com/Kandil7/kandil_code/releases for the draft release."