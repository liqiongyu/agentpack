## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared evolve.restore JSON data construction
- [x] Run `openspec validate refactor-app-evolve-restore-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared evolve.restore JSON data builder under `src/app/`
- [x] Update CLI evolve restore and MCP evolve_restore to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
