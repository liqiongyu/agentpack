## 1. Spec
- [x] Update OpenSpec delta for `agentpack-cli` to require `supports_json`, `args[]`, and `global_args[]` in `help --json`.

## 2. Implementation
- [x] Generate `help --json` command list from clap command structure (avoid hard-coded drift).
- [x] Include per-command argument metadata (`args[]`) excluding global args and built-in `--help/--version`.
- [x] Include `supports_json` per command (`completions=false`, others true).
- [x] Add `global_args[]` describing global CLI args.
- [x] Update `schema --json` to document help payload fields (additive).

## 3. Tests
- [x] Extend `tests/cli_help_schema.rs` to assert `supports_json` exists and `global_args` is present.

## 4. Docs
- [x] Update `docs/JSON_API.md` with a short section describing `help --json` / `schema` self-description payloads.

## 5. Validation
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run `cargo test --all --locked`
- [x] Run `openspec validate update-help-json-metadata --strict --no-interactive`
