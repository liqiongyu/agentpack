## 1. Implementation

- [ ] 1.1 Add `src/handlers/doctor.rs` for doctor report computation.
- [ ] 1.2 Refactor CLI `doctor` to use the handler (behavior preserved).

## 2. Spec deltas

- [ ] 2.1 Add a delta requirement describing doctor handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [ ] 3.1 `openspec validate refactor-command-handlers-doctor --strict --no-interactive`
- [ ] 3.2 `cargo fmt --all -- --check`
- [ ] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 3.4 `cargo test --all --locked`
