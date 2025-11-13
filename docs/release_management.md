# Release Management Guide for Kandil Code

This guide documents the complete process for releasing new versions of Kandil Code, including setup, procedures, and best practices.

## Prerequisites

Before starting the release process, ensure you have:

1. **Rust and Cargo** installed (verified with `rustc --version` and `cargo --version`)
2. **Git** properly configured with your GitHub credentials
3. **GitHub CLI** installed (optional, but helpful for managing releases)
4. **Access** to the repository with appropriate permissions
5. **All tests passing** on the main branch

## Release Strategy

Kandil Code follows semantic versioning and a structured release process:

- **Major releases** (x.0.0): Major features, breaking changes
- **Minor releases** (x.x.0): New features, enhancements
- **Patch releases** (x.x.x): Bug fixes, security patches

## Pre-Release Checklist

Before starting a release, verify:

- [ ] All features for the milestone are completed and tested
- [ ] All issues in the milestone are closed or moved to a future milestone
- [ ] All tests pass (run `cargo test`)
- [ ] The project builds successfully (run `cargo build --release`)
- [ ] Code formatting is consistent (run `cargo fmt`)
- [ ] Lint checks pass (run `cargo clippy`)
- [ ] Documentation is up to date
- [ ] The `CHANGELOG.md` file is updated with all changes
- [ ] The `README.md` file reflects the new features if applicable
- [ ] The `Cargo.toml` version is ready to be updated

## The Release Process

### Step 1: Prepare the Release Branch

1. Ensure your main branch is up to date:
   ```bash
   git checkout main
   git pull origin main
   ```

2. Create a release branch (optional but recommended for complex releases):
   ```bash
   git checkout -b release-vX.Y.Z
   ```

### Step 2: Update Version Information

1. Update the version in `Cargo.toml`:
   ```toml
   [package]
   name = "kandil_code"
   version = "X.Y.Z"  # Update to the new version
   ```

2. Update the version badge in `README.md` if present:
   ```markdown
   [![Version](https://img.shields.io/badge/Version-vX.Y.Z-blue)](https://github.com/Kandil7/kandil_code/releases)
   ```

### Step 3: Update Changelog

1. Update `CHANGELOG.md` with the changes for this release:
   - Add the new version under the "Unreleased" section
   - Move all unreleased changes to the new version section
   - Add the release date (today's date)
   - Add a link to the GitHub comparison URL

### Step 4: Code Quality Checks

Run the following commands to ensure code quality:

```bash
# Format the code
cargo fmt

# Run linters
cargo clippy -- -D warnings

# Run all tests
cargo test

# Build the release binary
cargo build --release
```

### Step 5: Commit and Push Changes

```bash
git add .
git commit -m "Prepare for release v.X.Y.Z"
git push origin release-vX.Y.Z  # if using release branch
```

### Step 6: Create Pull Request (if using release branch)

1. Create a pull request from your release branch to main
2. Have the changes reviewed by other maintainers
3. Merge the pull request once approved

### Step 7: Tag and Release

1. Create an annotated git tag:
   ```bash
   git tag -a vX.Y.Z -m "Release version X.Y.Z"
   ```

2. Push the tag to GitHub:
   ```bash
   git push origin vX.Y.Z
   ```

### Step 8: Automated Release Process

Once the tag is pushed:

1. GitHub Actions will automatically:
   - Build the project for all supported platforms
   - Run tests to ensure quality
   - Package binaries for distribution
   - Create a draft GitHub release

2. The release workflow will:
   - Create a GitHub release with the new tag
   - Attach platform-specific binaries
   - Mark the release as public

### Step 9: Verify the Release

1. Go to the [GitHub Releases page](https://github.com/Kandil7/kandil_code/releases)
2. Verify the new release exists with correct information
3. Check that all binaries are attached and downloadable
4. Verify the release notes match the changelog
5. Test downloading and running the binary on a clean system

### Step 10: Post-Release Tasks

1. Update the documentation site if applicable
2. Announce the release on relevant channels
3. Update any external references to the project version
4. Create a new "next" milestone in GitHub Issues
5. Close the previous milestone

## Automated Release Scripts

Kandil Code includes release scripts in the `scripts/` directory to help automate parts of the release process:

- `scripts/release.sh` - For Unix-like systems
- `scripts/release.bat` - For Windows systems

To use the release script:

```bash
# For Unix
./scripts/release.sh X.Y.Z

# For Windows
scripts\release.bat X.Y.Z
```

## Handling Release Issues

If problems occur during the release process:

1. **Tag pushed incorrectly**: You can delete a tag with:
   ```bash
   git tag -d vX.Y.Z
   git push --delete origin vX.Y.Z
   ```

2. **Release needs changes**: Create a patch release rather than trying to fix the current one

3. **Failed CI/CD**: Check the GitHub Actions logs and fix any issues, then create a new release

## Security Considerations

- Never commit API keys or secrets to the repository
- Ensure all dependencies are up to date and secure
- Verify that the release workflow doesn't expose sensitive information
- Review all changes included in the release for security implications

## Maintainer Guidelines

- Always use semantic versioning
- Maintain clear and consistent changelog entries
- Test releases on multiple platforms before finalizing
- Keep the release process documentation updated
- Ensure other maintainers know the release process