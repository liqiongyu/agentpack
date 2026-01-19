## 1. Implementation

- [ ] Add Journey J1 integration test covering: init → update → preview --diff → deploy --apply → status → rollback.

## 2. Spec deltas

- [ ] Add a delta requirement describing Journey J1 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [ ] `openspec validate add-journey-j1-from-scratch --strict`
- [ ] `cargo test --all --locked`
