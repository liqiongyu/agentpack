## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP explain tool implementation
- [x] Run `openspec validate refactor-mcp-explain-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract `call_explain_in_process` into `src/mcp/tools/explain.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/commands)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
