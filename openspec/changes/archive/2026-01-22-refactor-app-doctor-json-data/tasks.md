## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared doctor JSON data construction
- [x] Run `openspec validate refactor-app-doctor-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared doctor JSON data builder under `src/app/`
- [x] Update CLI doctor and MCP doctor to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
