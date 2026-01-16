## 1. Contract (M1-E1-T2 / #307)
- [x] Define deterministic mapping rules in OpenSpec delta
- [x] Update `docs/SPEC.md` / `docs/JSON_API.md` if contract fields/semantics change
- [x] Run `openspec validate update-import-mapping-rules --strict --no-interactive`

## 2. Implementation
- [x] Update `import` module id mapping to avoid user/project collisions
- [x] Add tool tags (`codex`, `claude_code`, `cursor`, `vscode`) and scope tags (`user`, `project`) consistently
- [x] Keep plan ordering deterministic

## 3. Tests
- [x] Extend `tests/cli_import.rs` to assert mapping rules (ids/tags/targets) deterministically

## 4. Docs
- [x] Add 3 examples (repo-only / user-only / mixed) to `docs/WORKFLOWS.md`

## 5. Archive
- [x] After shipping: `openspec archive update-import-mapping-rules --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
