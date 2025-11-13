# Release Process for Kandil Code

This document outlines the process for creating releases of the Kandil Code project.

## Versioning

Kandil Code follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html):

- **MAJOR** version: Incompatible API changes (when we have a library component)
- **MINOR** version: Backward-compatible functionality additions
- **PATCH** version: Backward-compatible bug fixes

## Prerequisites

Before creating a release, ensure:

1. All tests pass
2. The `CHANGELOG.md` is updated with changes for the release
3. The `Cargo.toml` file has the correct version
4. The `README.md` has accurate information

## Release Process

### 1. Update Version Numbers

Update the version in `Cargo.toml`:

```toml
[package]
name = "kandil_code"
version = "0.1.0"  # Update this to the new version
```

### 2. Update Changelog

Make sure `CHANGELOG.md` is updated with all notable changes for the release.

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "Prepare for release v0.1.0"
```

### 4. Create Git Tag

```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
```

### 5. Push Changes and Tags

```bash
git push origin main
git push origin v0.1.0
```

### 6. Create GitHub Release

GitHub Actions will automatically create a draft release when a tag is pushed. Follow these steps:

1. Go to the [Releases page](https://github.com/Kandil7/kandil_code/releases)
2. Edit the draft release for your tag
3. Fill in the release title (e.g., "v0.1.0")
4. Copy the changelog entries for this release in the description
5. Publish the release

## Automated Release Process

When a new tag matching the pattern `v*` is pushed, GitHub Actions will:

1. Build the project for multiple platforms (Linux, macOS, Windows)
2. Run tests to ensure quality
3. Package binaries for each platform
4. Create a GitHub release with the binaries attached
5. (If applicable) Publish to crates.io (only for library crates)

## Preparing Release Assets

For each platform, the release will include:

- Linux: `kandil_code-linux-x86_64`
- macOS: `kandil_code-macos-x86_64` or `kandil_code-macos-arm64`
- Windows: `kandil_code-windows-x86_64.exe`

## Verification

After the release is complete:

1. Verify that all binaries are available for download
2. Test that the binaries work correctly on each platform
3. Check that the crates.io package (if published) works as expected
4. Verify that documentation is updated on docs.rs (if applicable)

## Post-Release

After a successful release:

1. Update the project's README with the latest version badge
2. Announce the release on relevant channels
3. Update any documentation sites if applicable