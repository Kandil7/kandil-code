# CI/CD Pipeline Documentation

Kandil Code uses GitHub Actions for continuous integration and deployment. The pipeline ensures code quality, security, and automated releases.

## CI Pipeline

The CI pipeline runs on every push and pull request to ensure code quality:

### Test Matrix
- **Platforms**: Ubuntu, Windows, macOS
- **Rust versions**: Stable
- **Tests**: Formatting, linting, unit/integration tests

### Security Checks
- Dependency auditing with `cargo audit`
- License and security vulnerability scanning with `cargo-deny`
- Test coverage analysis

### Mobile Builds
- Cross-compilation for Android targets:
  - aarch64-linux-android (ARM64)
  - armv7-linux-androideabi (ARMv7)
  - x86_64-linux-android
  - i686-linux-android

## Release Pipeline

The release pipeline is triggered on version tags (`v*`) and includes:

### Desktop Builds
- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- x86_64-apple-darwin
- aarch64-apple-darwin
- x86_64-pc-windows-msvc
- aarch64-pc-windows-msvc

### Mobile Builds
- Android library builds (.so files) for multiple architectures
- Packaged for distribution with checksums

## Security Gates

The pipeline includes several security gates:

1. **Dependency Audit**: All dependencies are checked against the RustSec advisory database
2. **License Compliance**: Dependency licenses are validated using cargo-deny
3. **Code Scanning**: Automated security scanning of the codebase
4. **Signing**: Release artifacts are signed for integrity verification

## Configuration

### Workflows
- `ci.yml`: Main CI pipeline with matrix builds
- `release.yml`: Automated release workflow
- `crates_publish.yml`: Rust crate publishing
- `publish.yml`: Additional publishing tasks
- Various specialized workflows for different purposes

### Artifacts
- Binaries for all supported platforms
- Mobile libraries for Android integration
- Checksums for integrity verification
- Documentation and release notes

## Testing Strategy

### Unit Tests
- Core functionality testing
- CLI command validation
- AI integration tests

### Integration Tests
- End-to-end workflow validation
- Cross-platform compatibility
- Performance benchmarks

### Security Tests
- Input validation
- Secure credential handling
- Dependency scanning