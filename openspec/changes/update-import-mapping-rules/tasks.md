## 1. Contract (M1-E1-T2 / #307)
- [ ] Define deterministic mapping rules in OpenSpec delta
- [ ] Update `docs/SPEC.md` / `docs/JSON_API.md` if contract fields/semantics change
- [ ] Run `openspec validate update-import-mapping-rules --strict --no-interactive`

## 2. Implementation
- [ ] Update `import` module id mapping to avoid user/project collisions
- [ ] Add tool tags (`codex`, `claude_code`, `cursor`, `vscode`) and scope tags (`user`, `project`) consistently
- [ ] Keep plan ordering deterministic

## 3. Tests
- [ ] Extend `tests/cli_import.rs` to assert mapping rules (ids/tags/targets) deterministically

## 4. Docs
- [ ] Add 3 examples (repo-only / user-only / mixed) to `docs/WORKFLOWS.md`

## 5. Archive
- [ ] After shipping: `openspec archive update-import-mapping-rules --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
