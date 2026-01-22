## 1. Spec & planning
- [x] Add OpenSpec delta requirements for extracting MCP doctor tool implementation
- [x] Run `openspec validate refactor-mcp-doctor-tool-module --strict --no-interactive`

## 2. Implementation
- [x] Extract `call_doctor_in_process` into `src/mcp/tools/doctor.rs`
- [x] Keep `src/mcp/tools.rs` behavior unchanged (including errors/envelope/commands)

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
