## 1. Contract (M1-E1-T1 / #306)
- [x] Define CLI flags + behavior (dry-run default, `--apply`, `--home-root`)
- [x] Define `--json` payload for `command="import"` and update `docs/JSON_API.md` + `docs/SPEC.md`
- [x] Run `openspec validate add-import-command --strict --no-interactive`

## 2. Implementation
- [x] Add CLI wiring + `import` handler
- [x] Implement repo + home scanners (read-only)
- [x] Materialize imported assets into config repo modules (atomic writes)
- [x] Update `agentpack.yaml` with new modules (idempotent; deterministic ordering)

## 3. Tests
- [x] Integration fixtures (temp repo + temp home): dry-run plan stability
- [x] Integration fixtures: `--apply` writes modules + updates manifest

## 4. Docs
- [x] `docs/CLI.md`: add `import` section
- [x] `docs/WORKFLOWS.md`: add “migrate existing environment” workflow

## 5. Archive
- [x] After shipping: `openspec archive add-import-command --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
