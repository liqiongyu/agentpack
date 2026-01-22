## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP deploy_apply tool implementation
- [x] Run `openspec validate refactor-mcp-deploy-apply-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract the `deploy_apply` MCP tool handler into `src/mcp/tools/deploy_apply.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including confirm token enforcement and errors/envelope)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
