## 1. Implementation

- [x] 1.1 Add `src/handlers/read_only.rs` for shared planning pipeline.
- [x] 1.2 Update CLI `plan`/`diff`/`preview`/`deploy` to use the handler.
- [x] 1.3 Update `tui_core` read-only views to use the handler.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing read-only handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-command-handlers-readonly --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
