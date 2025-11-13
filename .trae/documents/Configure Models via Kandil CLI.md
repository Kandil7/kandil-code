## Current Behavior
- Local provider `ollama` selected via `kandil switch-model ollama <model>` (non-persistent).
- Active provider/model read from env vars in `src/utils/config.rs:46-55`.
- No validation of Ollama availability; no local model discovery or management.

## Enhancements
- Add local-model management commands under a focused subcommand:
  - `kandil local-model list` — list installed Ollama models.
  - `kandil local-model pull <model>` — download a model (with progress).
  - `kandil local-model remove <model>` — uninstall a local model.
  - `kandil local-model use <model>` — set current local model (persists selection).
  - `kandil local-model status` — check Ollama daemon availability and current selection.
- Persist selection:
  - Stage 1: persist via env (`KANDIL_AI_PROVIDER=ollama`, `KANDIL_AI_MODEL=<model>`), updated automatically by `use`.
  - Stage 2 (optional if approved): write to `kandil.toml` and read in `Config::load`, keeping env override.
- Validation improvements:
  - Extend `Config::validate_production` (`src/utils/config.rs:62-79`) to ping Ollama at `http://localhost:11434` and verify the selected model exists.
  - Provide actionable errors when Ollama is not running or model is missing.

## Implementation Details
- CLI wiring:
  - Add `Commands::LocalModel { sub: LocalModelSub }` in `src/cli/mod.rs` near `ConfigSub` (`src/cli/mod.rs:632-648`).
  - Implement handlers: `handle_local_model(sub)` analogous to `handle_config` (`src/cli/mod.rs:1365-1399`).
- Ollama API integration (reuse `reqwest`):
  - `list`: GET `http://localhost:11434/api/tags` → parse names.
  - `pull`: POST `http://localhost:11434/api/pull` with `{ "name": "<model>" }` → show progress messages.
  - `remove`: POST `http://localhost:11434/api/delete` with `{ "name": "<model>" }`.
  - `status`: GET `http://localhost:11434/` or attempt a lightweight call to `/api/tags`.
  - Add a new helper module `utils::ollama` with small functions to encapsulate these calls.
- Persistence behavior:
  - `use <model>` sets provider to `ollama` and updates env vars in-process; optionally write to TOML in Stage 2.
- Tests:
  - Extend `tests/cli.rs` to assert helpful failure messages when Ollama is unavailable, and success paths for `status`.
  - Unit tests for JSON request/response shapes without hitting the network.

## Code References
- Current `switch-model` handler: `src/cli/mod.rs:1326-1335`.
- Config loader and validation: `src/utils/config.rs:46-55`, `src/utils/config.rs:62-79`.
- AI adapter base (Ollama URL, generate endpoint): `src/core/adapters/ai/mod.rs:40-45`, `src/core/adapters/ai/mod.rs:64-100`.

## Rollout
- Implement CLI subcommand and helpers.
- Wire validation and persistence.
- Add tests and messages.
- Document example flows in help output.

## Confirmation
- If approved, I will add `local-model` commands, Ollama API integration, and improved validation/persistence focused on local models, without introducing new dependencies.