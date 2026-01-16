## 1. Contract (M1-E1-T1 / #306)
- [ ] Define CLI flags + behavior (dry-run default, `--apply`, `--home-root`)
- [ ] Define `--json` payload for `command="import"` and update `docs/JSON_API.md` + `docs/SPEC.md`
- [x] Run `openspec validate add-import-command --strict --no-interactive`

## 2. Implementation
- [ ] Add CLI wiring + `import` handler
- [ ] Implement repo + home scanners (read-only)
- [ ] Materialize imported assets into config repo modules (atomic writes)
- [ ] Update `agentpack.yaml` with new modules (idempotent; deterministic ordering)

## 3. Tests
- [ ] Integration fixtures (temp repo + temp home): dry-run plan stability
- [ ] Integration fixtures: `--apply` writes modules + updates manifest

## 4. Docs
- [ ] `docs/CLI.md`: add `import` section
- [ ] `docs/WORKFLOWS.md`: add “migrate existing environment” workflow

## 5. Archive
- [ ] After shipping: `openspec archive add-import-command --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
