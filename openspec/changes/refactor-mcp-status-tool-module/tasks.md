## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP status tool implementation
- [x] Run `openspec validate refactor-mcp-status-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract `call_status_in_process` into `src/mcp/tools/status.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/commands)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
