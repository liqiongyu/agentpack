## 1. Docs & specs
- [x] Update `docs/SPEC.md` target list and add a `cursor` target section
- [x] Update `docs/CLI.md`, `docs/CONFIG.md`, `docs/TARGETS.md` (and `docs/zh-CN/*`) to document `cursor`
- [x] Add OpenSpec delta requirements under `openspec/changes/add-cursor-target/specs/`

## 2. Implementation
- [x] Allow `cursor` in manifest validation and CLI `--target` selection
- [x] Implement `cursor` `TargetAdapter` and `Engine::render_cursor` mapping to `.cursor/rules/*.mdc`
- [x] Add conformance coverage (`deploy/status/rollback`) for `cursor`

## 3. Validation
- [x] Run: `cargo fmt --all -- --check`
- [x] Run: `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run: `cargo test --all --locked`
- [x] Run: `openspec validate add-cursor-target --strict --no-interactive`
