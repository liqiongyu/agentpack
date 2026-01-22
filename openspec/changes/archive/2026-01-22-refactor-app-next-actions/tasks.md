## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared next_actions ordering
- [x] Run `openspec validate refactor-app-next-actions --strict --no-interactive`

## 2. Implementation
- [x] Add `src/app/next_actions.rs`
- [x] Update CLI status and MCP status to use shared helpers

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
