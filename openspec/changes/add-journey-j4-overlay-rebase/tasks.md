## 1. Implementation

- [x] Add Journey J4 integration test covering sparse overlay creation, materialize, rebase conflict behavior, and deploy using overlay-composed desired state.

## 2. Spec deltas

- [x] Add a delta requirement describing Journey J4 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] `openspec validate add-journey-j4-overlay-rebase --strict`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
