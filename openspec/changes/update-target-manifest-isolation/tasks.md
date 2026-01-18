## 1. Contract (#405)
- [x] Update OpenSpec deltas for target manifest naming + backwards compatibility
- [x] Run `openspec validate update-target-manifest-isolation --strict --no-interactive`

## 2. Implementation
- [ ] Write manifests as `<root>/.agentpack.manifest.<target>.json`
- [ ] Read legacy `.agentpack.manifest.json` only when `tool == <target>`
- [ ] Update rollback manifest restore logic to cover new filenames
- [ ] Update `doctor --fix` / `init --git` to ignore `.agentpack.manifest*.json`

## 3. Tests
- [ ] Update conformance + unit/integration tests that assert manifest filenames

## 4. Docs
- [ ] Update `docs/SPEC.md` (target manifest section) and any other relevant docs

## 5. Archive
- [ ] After shipping: `openspec archive update-target-manifest-isolation --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
