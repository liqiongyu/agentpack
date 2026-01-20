## 1. Implementation

- [x] 1.1 Add an in-process MCP path to compute the `deploy` plan JSON envelope (like `agentpack deploy --json` without `--apply`).
- [x] 1.2 Preserve confirm token behavior (`confirm_plan_hash` computed from binding + envelope data; token + expiry injected into the envelope).
- [x] 1.3 Update MCP tool dispatch for `deploy` to use the in-process path (no subprocess).

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing MCP `deploy` in-process execution (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-mcp-tools-deploy-handler --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
