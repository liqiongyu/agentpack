## 1. Implementation
- [x] Add `agentpack evolve restore` subcommand and wire it in CLI dispatch.
- [x] Implement restore behavior: write only missing desired outputs (create-only).
- [x] `--json` mode requires `--yes` (`E_CONFIRM_REQUIRED`) and outputs a stable summary.

## 2. Tests
- [x] Add a CLI test for `evolve restore --json` requiring `--yes`.
- [x] Add a CLI test that verifies `evolve restore` recreates a missing desired output file.

## 3. Docs
- [x] Update `docs/SPEC.md` to document `evolve restore` behavior and safety properties.

## 4. Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.
- [x] Run `openspec validate add-evolve-restore --strict --no-interactive`.
