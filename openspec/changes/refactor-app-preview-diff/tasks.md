## 1. Spec & planning
- [x] Add OpenSpec delta requirements for shared preview diff generation
- [x] Run `openspec validate refactor-app-preview-diff --strict --no-interactive`

## 2. Implementation
- [x] Add shared `best_root_idx` helper (dedupe duplicates)
- [x] Add `src/app/preview_diff.rs` and wire it into CLI + MCP preview implementations

## 3. Verification
- [x] `cargo fmt --all`
- [x] `just check`
