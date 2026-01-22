## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared status JSON data construction
- [x] Run `openspec validate refactor-app-status-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared status JSON data builder under `src/app/`
- [x] Update CLI status and MCP status to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
