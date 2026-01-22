## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared status drift filtering
- [x] Run `openspec validate refactor-app-status-drift-filter --strict --no-interactive`

## 2. Implementation
- [x] Add shared drift filtering helper under `src/app/status_drift.rs`
- [x] Update CLI status and MCP status to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
