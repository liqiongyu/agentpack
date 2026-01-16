## 1. Contract (M1-E4-T1 / #317)
- [x] Define the JSON golden snapshot coverage requirement
- [x] Run `openspec validate expand-cli-json-golden-snapshots --strict --no-interactive`

## 2. Implementation
- [ ] Add missing JSON golden snapshots for `init` and `update` success paths
- [ ] Add missing JSON golden snapshots for `plan`, `diff`, and `preview` (non-diff) success paths
- [ ] Add missing JSON golden snapshots for `overlay path` and `evolve` (representative deterministic scenarios)

## 3. Tests
- [ ] Ensure snapshots are deterministic across platforms (path normalization + stable placeholders)
- [ ] Ensure `cargo test --all --locked` runs the golden suite in CI

## 4. Archive
- [ ] After shipping: `openspec archive expand-cli-json-golden-snapshots --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
