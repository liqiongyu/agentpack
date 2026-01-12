## 1. Implementation
- [x] Map common failures to stable `UserError` codes.
- [x] Ensure CLI JSON output uses those codes.

## 2. Tests
- [x] Add CLI tests for config/lockfile/target error codes.

## 3. Docs
- [x] Update `docs/SPEC.md` with the stable error codes list and examples.

## 4. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-5-json-error-codes --strict --no-interactive`.
