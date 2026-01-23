---
## 1. Implementation
- [x] Add shared helper for extracting `UserError` from `anyhow::Error`
- [x] Refactor CLI + MCP call sites to use the shared helper (no behavior change)

## 2. Verification
- [x] `openspec validate refactor-anyhow-user-error-helper --strict --no-interactive`
- [x] `cargo fmt --all`
- [x] `just check`
