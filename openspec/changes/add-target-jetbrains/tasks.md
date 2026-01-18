## 1. Contract (M3-E2 / #392)
- [x] Define target behavior in OpenSpec deltas (`agentpack`, `agentpack-cli`)
- [x] Run `openspec validate add-target-jetbrains --strict --no-interactive`

## 2. Implementation
- [ ] Add Cargo feature `target-jetbrains` (and include in default features)
- [ ] Register the `jetbrains` target in the target registry and config validation
- [ ] Implement the `jetbrains` TargetAdapter (render `.junie/guidelines.md`)

## 3. Tests
- [ ] Add conformance coverage for `jetbrains` target
- [ ] Update CI conformance feature matrix to include `target-jetbrains`
- [ ] Update JSON/golden snapshots if `help --json` changes

## 4. Docs
- [ ] Document JetBrains target mapping + examples + migration notes

## 5. Archive
- [ ] After shipping: `openspec archive add-target-jetbrains --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
