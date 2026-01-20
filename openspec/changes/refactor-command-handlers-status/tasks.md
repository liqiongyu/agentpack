## 1. Implementation

- [ ] 1.1 Add `src/handlers/status.rs` for status drift computation (manifests + drift + warnings).
- [ ] 1.2 Refactor CLI `status` to use the handler for drift computation (behavior preserved).
- [ ] 1.3 Refactor TUI status rendering to reuse the handler.

## 2. Spec deltas

- [ ] 2.1 Add a delta requirement describing status drift handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [ ] 3.1 `openspec validate refactor-command-handlers-status --strict --no-interactive`
- [ ] 3.2 `cargo fmt --all -- --check`
- [ ] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 3.4 `cargo test --all --locked`
