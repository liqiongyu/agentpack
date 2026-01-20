## 1. Implementation

- [x] 1.1 Add `src/handlers/evolve.rs` (or equivalent) for `evolve.restore` logic + guardrails.
- [x] 1.2 Refactor CLI `evolve restore` to use the handler.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing evolve.restore handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-command-handlers-evolve-restore --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
