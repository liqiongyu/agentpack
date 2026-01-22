## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared explain JSON data construction
- [x] Run `openspec validate refactor-app-explain-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared explain JSON data builders under `src/app/`
- [x] Update CLI explain and MCP explain to use them

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
