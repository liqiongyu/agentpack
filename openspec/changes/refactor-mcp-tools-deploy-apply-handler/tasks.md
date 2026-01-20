## 1. Implementation

- [x] 1.1 Add an in-process MCP path to execute `deploy --apply` logic for `deploy_apply` (no subprocess) while keeping confirm-token validation.
- [x] 1.2 Preserve CLI-style `deploy --apply --json` envelope fields (`applied`, `snapshot_id`, `reason`, `changes`, `summary`) and stable error codes.
- [x] 1.3 Update MCP tool dispatch for `deploy_apply` to use the in-process apply path.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing MCP `deploy_apply` in-process execution (archive with `--skip-specs` since this is refactor-only).

## 3. Validation

- [x] 3.1 `openspec validate refactor-mcp-tools-deploy-apply-handler --strict --no-interactive`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
