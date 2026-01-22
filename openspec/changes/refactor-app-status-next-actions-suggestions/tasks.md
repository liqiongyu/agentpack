## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared status next_actions suggestions
- [x] Run `openspec validate refactor-app-status-next-actions-suggestions --strict --no-interactive`

## 2. Implementation
- [x] Add shared status next_actions suggestion helper under `src/app/`
- [x] Update CLI status and MCP status to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
