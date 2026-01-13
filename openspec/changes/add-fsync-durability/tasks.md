## 1. Implementation
- [x] Add `AGENTPACK_FSYNC` opt-in behavior to `write_atomic`.

## 2. Tests
- [x] Add regression coverage for `AGENTPACK_FSYNC=1` write paths.

## 3. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate add-fsync-durability --strict --no-interactive`.
