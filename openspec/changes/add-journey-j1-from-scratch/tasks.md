## 1. Implementation

- [x] Add Journey J1 integration test covering: init → update → preview --diff → deploy --apply → status → rollback.

## 2. Spec deltas

- [x] Add a delta requirement describing Journey J1 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] `openspec validate add-journey-j1-from-scratch --strict`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
