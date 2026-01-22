## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared evolve.propose JSON data construction
- [x] Run `openspec validate refactor-app-evolve-propose-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared evolve.propose JSON data builder under `src/app/`
- [x] Update CLI evolve propose and MCP evolve_propose to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
