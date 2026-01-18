## 1. Contract (#405)
- [x] Update OpenSpec deltas for target manifest naming + backwards compatibility
- [x] Run `openspec validate update-target-manifest-isolation --strict --no-interactive`

## 2. Implementation
- [x] Write manifests as `<root>/.agentpack.manifest.<target>.json`
- [x] Read legacy `.agentpack.manifest.json` only when `tool == <target>`
- [x] Update rollback manifest restore logic to cover new filenames
- [x] Update `doctor --fix` / `init --git` to ignore `.agentpack.manifest*.json`

## 3. Tests
- [x] Update conformance + unit/integration tests that assert manifest filenames

## 4. Docs
- [x] Update `docs/SPEC.md` (target manifest section) and any other relevant docs

## 5. Archive
- [x] After shipping: `openspec archive update-target-manifest-isolation --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
