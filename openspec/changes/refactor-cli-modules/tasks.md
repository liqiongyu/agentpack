## 1. Implementation
- [x] Create `src/cli/` module scaffold and move `run()` + CLI argument parsing.
- [x] Move JSON envelope + error mapping helpers into `src/cli/json.rs`.
- [x] Move command handlers into `src/cli/commands/` modules (one command per file where practical).
- [x] Keep behavior stable: same default outputs, same `--json` envelopes, same error codes.

## 2. Tests
- [x] Run existing CLI integration tests and goldens; update only if behavior changes are intended (they are not).

## 3. Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.
- [x] Run `openspec validate refactor-cli-modules --strict --no-interactive`.
