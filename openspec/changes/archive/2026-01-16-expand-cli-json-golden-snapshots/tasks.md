## 1. Contract (M1-E4-T1 / #317)
- [x] Define the JSON golden snapshot coverage requirement
- [x] Run `openspec validate expand-cli-json-golden-snapshots --strict --no-interactive`

## 2. Implementation
- [x] Add missing JSON golden snapshots for `init` and `update` success paths
- [x] Add missing JSON golden snapshots for `plan`, `diff`, and `preview` (non-diff) success paths
- [x] Add missing JSON golden snapshots for `overlay path` and `evolve` (representative deterministic scenarios)

## 3. Tests
- [x] Ensure snapshots are deterministic across platforms (path normalization + stable placeholders)
- [x] Ensure `cargo test --all --locked` runs the golden suite in CI

## 4. Archive
- [x] After shipping: `openspec archive expand-cli-json-golden-snapshots --yes`
- [x] Run `openspec validate --all --strict --no-interactive`
