## 1. Implementation

- [ ] Add Journey J2 integration test covering import dry-run vs apply (user + project assets) and post-import preview/deploy.

## 2. Spec deltas

- [ ] Add a delta requirement describing Journey J2 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [ ] `openspec validate add-journey-j2-import --strict`
- [ ] `cargo test --all --locked`
