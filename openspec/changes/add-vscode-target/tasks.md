## 1. Docs & specs
- [x] Update `docs/SPEC.md` target list and add a `vscode` target section
- [x] Update `docs/CLI.md`, `docs/CONFIG.md`, `docs/TARGETS.md` (and `docs/zh-CN/*`) to document `vscode`
- [x] Add OpenSpec delta requirements under `openspec/changes/add-vscode-target/specs/`

## 2. Implementation
- [x] Allow `vscode` in manifest/module validation and CLI `--target` selection
- [x] Implement `vscode` `TargetAdapter` and `Engine::render_vscode` mapping to `.github/`
- [x] Add conformance coverage (`deploy/status/rollback`) for `vscode`

## 3. Validation
- [x] Run: `cargo fmt --all -- --check`
- [x] Run: `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run: `cargo test --all --locked`
- [x] Run: `openspec validate add-vscode-target --strict --no-interactive`
