## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared doctor next_actions suggestions
- [x] Run `openspec validate refactor-app-doctor-next-actions-suggestions --strict --no-interactive`

## 2. Implementation
- [x] Add shared doctor next_actions suggestion helper under `src/app/`
- [x] Update CLI doctor and MCP doctor to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
