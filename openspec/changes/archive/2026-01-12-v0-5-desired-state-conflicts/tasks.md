## 1. Implementation
- [x] Add conflict detection when inserting into DesiredState.
- [x] Ensure `--json` surfaces a stable error code for conflicts.

## 2. Tests
- [x] Add a CLI test that reproduces a conflicting output and asserts the JSON error code.

## 3. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-5-desired-state-conflicts --strict --no-interactive`.
