## Naming & Install Alignment

* Keep crate `kandil_code`; expose binary as `kandil` via Cargo bin settings.

* Ensure `--version`/`--help` are accurate and consistent in CLI.

## Release Workflow

* Add GitHub Actions workflow triggered by tags `v*`.

* Job A: Create release and changelog; output upload URL.

* Job B: Matrix builds for Linux/macOS (x86\_64, aarch64) and Windows (MSVC), cache Cargo, build `--release`, package (tar.gz/zip), generate checksums, upload assets.

## Optional npm Wrapper

* Scaffold `npm-wrapper` with `package.json`, `bin/kandil.js`, `scripts/download-binary.js`.

* Map `process.platform`/`arch` to release asset names; download, verify checksum, set executable bit, spawn binary.

* Provide clear errors and proxy support; basic tests around mapping and failures.

## Auth Commands

* Add `kandil auth login --provider <openai|claude|qwen>`.

* Prompt for key and store via existing keyring (`SecureKey`), with success/failure messaging.

* Keep environment variable override for non-interactive usage.

## Rate Limiting

* Integrate per-provider, per-key limiter with `CostTracker`.

* Default limits: 60 req/min; configurable via env vars.

* Call limiter in cloud adapters before requests; return actionable errors when exceeded.

## Documentation

* Update README install methods (Cargo recommended, npm optional, direct download with scripts).

* Quickstart aligned to actual commands (`init`, `chat`, `create`, `local-model`, `config validate`, `auth login`).

* Fix broken URLs (e.g., `githb.com` → `github.com`) and align release asset names.

## Tests

* Extend CLI tests: `auth login` non-interactive paths (use `config set-key`), limiter behavior, release asset naming helpers.

* Smoke tests for `--version` and help.

## Security

* Secrets stored in OS keyring only; never persisted in files.

* npm wrapper verifies checksums; fails closed on mismatch.

## Rollout

* Implement bin name and auth CLI.

* Add rate limiter and integrate with adapters.

* Add release workflow.

* Add optional npm wrapper scaffolding.

* Update docs and tests.

## Acceptance

* Install via Cargo produces `kandil` executable; `--help` works.

* Tagged releases build and upload assets with checksums for all targets.

* Optional npm wrapper installs and runs across platforms.

* Auth login stores keys and cloud providers work with limits.

* Docs updated and accurate.

