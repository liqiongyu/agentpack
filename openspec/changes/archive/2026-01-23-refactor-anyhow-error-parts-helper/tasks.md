---
## 1. Implementation
- [x] Add shared helper for mapping `anyhow::Error` into `(code, message, details)` for envelopes
- [x] Refactor CLI JSON + MCP tool envelope mapping to use the shared helper

## 2. Verification
- [x] `openspec validate refactor-anyhow-error-parts-helper --strict --no-interactive`
- [x] `cargo fmt --all`
- [x] `just check`
