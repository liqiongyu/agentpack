## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP evolve_restore tool implementation
- [x] Run `openspec validate refactor-mcp-evolve-restore-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract `call_evolve_restore_in_process` into `src/mcp/tools/evolve_restore.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/commands)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
