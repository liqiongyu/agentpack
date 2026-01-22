## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared deploy JSON data construction
- [x] Run `openspec validate refactor-app-deploy-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared deploy JSON data builder under `src/app/`
- [x] Update CLI deploy and MCP deploy/deploy_apply to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
