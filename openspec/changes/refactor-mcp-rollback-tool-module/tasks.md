## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP rollback tool implementation
- [x] Run `openspec validate refactor-mcp-rollback-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract `call_rollback_in_process` into `src/mcp/tools/rollback.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/commands)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
