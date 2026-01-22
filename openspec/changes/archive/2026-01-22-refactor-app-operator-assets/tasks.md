## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared operator assets checks
- [x] Run `openspec validate refactor-app-operator-assets --strict --no-interactive`

## 2. Implementation
- [x] Add `src/app/operator_assets.rs`
- [x] Update CLI status and MCP status to use shared helpers

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
