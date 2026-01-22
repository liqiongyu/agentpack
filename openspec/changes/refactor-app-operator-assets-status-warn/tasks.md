## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared status operator-assets warnings
- [x] Run `openspec validate refactor-app-operator-assets-status-warn --strict --no-interactive`

## 2. Implementation
- [x] Add shared operator-assets warning helper under `src/app/operator_assets.rs`
- [x] Update CLI status and MCP status to use it

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
