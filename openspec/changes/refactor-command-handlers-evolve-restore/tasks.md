## 1. Implementation

- [ ] 1.1 Add `src/handlers/evolve.rs` (or equivalent) for `evolve.restore` logic + guardrails.
- [ ] 1.2 Refactor CLI `evolve restore` to use the handler.

## 2. Spec deltas

- [ ] 2.1 Add a delta requirement describing evolve.restore handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [ ] 3.1 `openspec validate refactor-command-handlers-evolve-restore --strict --no-interactive`
- [ ] 3.2 `cargo fmt --all -- --check`
- [ ] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 3.4 `cargo test --all --locked`
