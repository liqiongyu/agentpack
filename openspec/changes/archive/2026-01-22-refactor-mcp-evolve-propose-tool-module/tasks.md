## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP evolve_propose tool implementation
- [x] Run `openspec validate refactor-mcp-evolve-propose-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract `call_evolve_propose_in_process` into `src/mcp/tools/evolve_propose.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/commands)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
