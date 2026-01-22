## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared status drift summary helpers
- [x] Run `openspec validate refactor-app-status-drift-summary --strict --no-interactive`

## 2. Implementation
- [x] Add `src/app/status_drift.rs`
- [x] Update CLI status and MCP status to use shared helpers

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
