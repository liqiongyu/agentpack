## 1. Implementation

- [x] 1.1 Add `src/handlers/deploy.rs` for shared deploy apply logic and guardrails.
- [x] 1.2 Move `manifests_missing_for_desired` logic into a non-CLI module (used by CLI/TUI/handlers).
- [x] 1.3 Refactor CLI `deploy --apply` to use the handler (preserve `--json` behavior).
- [x] 1.4 Refactor `tui_apply` to use the handler.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing deploy apply handler modularization (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-command-handlers-deploy-apply --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
