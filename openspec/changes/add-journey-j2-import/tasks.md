## 1. Implementation

- [x] Add Journey J2 integration test covering import dry-run vs apply (user + project assets) and post-import preview/deploy.

## 2. Spec deltas

- [x] Add a delta requirement describing Journey J2 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] `openspec validate add-journey-j2-import --strict`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
