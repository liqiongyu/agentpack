## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP deploy tool implementation
- [x] Run `openspec validate refactor-mcp-deploy-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract the `deploy` MCP tool handler into `src/mcp/tools/deploy.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/confirm tokens)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
