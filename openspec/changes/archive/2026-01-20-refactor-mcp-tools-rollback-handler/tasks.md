## 1. Implementation

- [x] 1.1 Add an in-process MCP path to compute the `rollback` JSON envelope via `src/handlers/rollback.rs`.
- [x] 1.2 Preserve CLI-style error envelope behavior (prefer `UserError` code/message/details when present).
- [x] 1.3 Update MCP tool dispatch for `rollback` to use the in-process path (no subprocess).

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing MCP `rollback` in-process execution (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-mcp-tools-rollback-handler --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
