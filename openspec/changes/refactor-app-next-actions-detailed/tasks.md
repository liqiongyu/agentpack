## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared status next_actions_detailed generation
- [x] Run `openspec validate refactor-app-next-actions-detailed --strict --no-interactive`

## 2. Implementation
- [x] Add shared next_actions_detailed helper under `src/app/next_actions.rs`
- [x] Update CLI status and MCP status to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
