## 1. Implementation
- [x] Remove pre-delete for create/update in apply paths.
- [x] Make `write_atomic` handle existing destinations (best-effort replace).

## 2. Tests
- [x] Ensure existing test suite passes (deploy/bootstrap/apply/rollback).

## 3. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-5-apply-atomic-update --strict --no-interactive`.
