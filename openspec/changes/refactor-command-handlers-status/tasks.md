## 1. Implementation

- [x] 1.1 Add `src/handlers/status.rs` for status drift computation (manifests + drift + warnings).
- [x] 1.2 Refactor CLI `status` to use the handler for drift computation (behavior preserved).
- [x] 1.3 Refactor TUI status rendering to reuse the handler.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing status drift handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-command-handlers-status --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
