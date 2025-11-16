# Publishing Kandil Code to Crates.io

This document outlines the complete process for publishing a new version of Kandil Code to crates.io.

## Prerequisites

1. **Crates.io Account**: You must have a crates.io account with publishing rights for the kandil_code crate
2. **API Token**: You need a valid crates.io API token
3. **Cargo Installation**: Ensure you have the latest stable Rust toolchain installed
4. **Git Repository**: Ensure all changes are committed to the git repository

## Pre-Publishing Checklist

Before publishing, ensure the following:

- [ ] Version number has been updated in `Cargo.toml`
- [ ] Version number has been updated in `release.toml`
- [ ] Changelog has been updated with `CHANGELOG.md`
- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt`
- [ ] Clippy warnings are addressed: `cargo clippy`
- [ ] The package builds without warnings: `cargo build --release`
- [ ] Documentation is up-to-date: `cargo doc --no-deps`
- [ ] The package passes dry run publish: `cargo publish --dry-run`
- [ ] All new features have been properly tested
- [ ] Breaking changes are documented
- [ ] Dependencies are properly specified with version ranges

## Authentication

Before publishing, you need to authenticate with crates.io:

```bash
# If you haven't authenticated before, get your API token from:
# https://crates.io/settings/tokens
cargo login YOUR_API_TOKEN_HERE
```

## Testing the Package

Before publishing, always test the package:

```bash
# Check for any issues before packaging
cargo check

# Run all tests
cargo test

# Format the code
cargo fmt

# Check for common issues
cargo clippy

# Create the package archive without publishing
cargo package

# Test publish dry-run (this simulates publishing without actually uploading)
cargo publish --dry-run
```

## Actual Publishing

Once everything is ready, publish the crate:

```bash
# Publish to crates.io
cargo publish
```

## Post-Publishing Steps

After successful publishing, you should:

1. **Tag the release in git**:
   ```bash
   git tag -a v2.1.0 -m "Release version 2.1.0"
   git push origin v2.1.0
   ```

2. **Create a GitHub release**:
   - Go to the GitHub releases page for the repository
   - Create a new release with the tag you just pushed
   - Use the changelog content for the release notes

3. **Verify the publication**:
   - Visit https://crates.io/crates/kandil_code to confirm the new version is live
   - Check that the documentation is building correctly

## Versioning Strategy

Kandil Code follows semantic versioning:

- **MAJOR.MINOR.PATCH** (e.g., 2.1.0)
- Increment the MAJOR version when making incompatible API changes
- Increment the MINOR version when adding functionality in a backward-compatible manner
- Increment the PATCH version for backward-compatible bug fixes

## Feature Flags

The published crate includes several optional features:
- `tui`: Enable terminal UI functionality (enabled by default)
- `gpu-rendering`: Enable GPU-accelerated rendering
- `wasm`: Enable WebAssembly support

Users can enable these features as needed when including Kandil Code as a dependency.

## Troubleshooting

Common publishing issues and resolutions:

### Authentication Issues
- Error: "Authentication error" or "API token required"
- Solution: Run `cargo login YOUR_API_TOKEN_HERE` again

### Dependency Issues
- Error: "Failed to verify package"
- Solution: Check all dependencies and their version ranges in Cargo.toml

### Size Issues
- Error: "Upload failed, too large"
- Solution: Check `Cargo.toml` for unnecessary files in `include` or remove large assets

### Dry Run Issues
- Always run `cargo publish --dry-run` first to catch issues
- Fix all errors/warnings before actual publishing

## Rollback Policy

Due to crates.io's immutability policy:
- Versions cannot be overwritten once published
- If a critical bug is found, publish a new patch version with the fix
- Major breaking issues may require a yanking (contact maintainers)

## Additional Notes

- The published crate will be available at: https://crates.io/crates/kandil_code
- Downloads and statistics are available on the crate page
- Users can depend on this crate in their `Cargo.toml`:
  ```
  [dependencies]
  kandil_code = "2.1.0"
  ```
- Feature-specific dependencies are only included when the respective feature is enabled