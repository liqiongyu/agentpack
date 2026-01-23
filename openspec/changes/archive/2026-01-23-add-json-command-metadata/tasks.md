## 1. Implementation

### 1.1 CLI command identification
- [x] Add `Cli::command_id()` and `Cli::command_path()` helpers.
- [x] Ensure mutating variants are represented as stable ids (`deploy --apply`, `doctor --fix`, `import --apply`).

### 1.2 JSON envelope fields
- [x] Add `command_id` and `command_path` to `JsonEnvelope` (additive).
- [x] Populate these fields for all success envelopes.
- [x] Populate these fields for the error envelope (`print_anyhow_error`).

### 1.3 Docs and schema
- [x] Update `agentpack schema --json` envelope field list.
- [x] Update `docs/SPEC.md` and `docs/reference/json-api.md`.

### 1.4 Tests
- [x] Update golden snapshot `tests/golden/schema_json_data.json`.
- [x] Add an integration test asserting `command_id`/`command_path` on an error path for a subcommand (e.g. `remote set --json` without `--yes`).

### 1.5 Validation
- [x] Run `openspec validate add-json-command-metadata --strict --no-interactive`.
- [x] Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all --locked`.
