## 1. Contract (M1-E5-T1 / #319)
- [x] Define the temp-roots conformance harness requirement
- [x] Run `openspec validate add-conformance-temp-roots-harness --strict --no-interactive`

## 2. Implementation
- [ ] Add a shared conformance harness that isolates all filesystem roots to temp dirs
- [ ] Ensure the harness sets deterministic env (no real home writes; parallel-safe)
- [ ] Refactor existing target conformance tests to use the harness

## 3. Tests
- [ ] `cargo test --all --locked` passes locally

## 4. Archive
- [ ] After shipping: `openspec archive add-conformance-temp-roots-harness --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
