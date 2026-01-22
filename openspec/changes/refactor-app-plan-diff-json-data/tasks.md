## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared plan/diff JSON data construction
- [x] Run `openspec validate refactor-app-plan-diff-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared plan/diff JSON data builder under `src/app/`
- [x] Update CLI plan/diff and MCP plan/diff to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
