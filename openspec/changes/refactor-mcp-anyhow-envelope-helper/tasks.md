---
## 1. Implementation
- [x] Add a shared helper to convert `anyhow::Error` -> Agentpack MCP error envelope (preserving `UserError` details)
- [x] Refactor selected read-only MCP tool handlers to use the helper (no behavior change)

## 2. Verification
- [x] `openspec validate refactor-mcp-anyhow-envelope-helper --strict --no-interactive`
- [x] `cargo fmt --all`
- [x] `just check`
