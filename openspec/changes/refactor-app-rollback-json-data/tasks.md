## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared rollback JSON data construction
- [x] Run `openspec validate refactor-app-rollback-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared rollback JSON data builder under `src/app/`
- [x] Update CLI rollback and MCP rollback to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
