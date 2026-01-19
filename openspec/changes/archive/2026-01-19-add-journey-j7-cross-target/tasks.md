## 1. Implementation

- [x] 1.1 Add Journey J7 integration test covering multi-target deploy + rollback.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing Journey J7 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] 3.1 `openspec validate add-journey-j7-cross-target --strict`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
