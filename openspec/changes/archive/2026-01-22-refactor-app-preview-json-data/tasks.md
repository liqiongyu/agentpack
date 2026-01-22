## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared preview JSON data construction
- [x] Run `openspec validate refactor-app-preview-json-data --strict --no-interactive`

## 2. Implementation
- [x] Add shared preview JSON data builder under `src/app/`
- [x] Update CLI preview and MCP preview to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
