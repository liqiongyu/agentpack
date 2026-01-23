---
## 1. Implementation
- [x] Refactor remaining MCP tools to use `envelope_from_anyhow_error()` for `anyhow::Error` -> envelope mapping

## 2. Verification
- [x] `openspec validate refactor-mcp-anyhow-envelope-helper-remaining-tools --strict --no-interactive`
- [x] `cargo fmt --all`
- [x] `just check`
