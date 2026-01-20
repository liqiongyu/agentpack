## 1. Implementation

- [x] 1.1 Add an in-process MCP path to compute `status` JSON envelopes via `src/handlers/status.rs`.
- [x] 1.2 Preserve CLI-style `status --json` fields (including `data.next_actions` and `data.next_actions_detailed`) and ordering.
- [x] 1.3 Update MCP tool dispatch for `status` to use the in-process path (no subprocess).

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing MCP `status` in-process execution (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-mcp-tools-status-handler --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
