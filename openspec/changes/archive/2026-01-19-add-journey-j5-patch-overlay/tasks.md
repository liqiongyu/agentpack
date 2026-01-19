## 1. Implementation

- [x] 1.1 Add Journey J5 integration test covering patch overlay creation, apply, and rebase conflict artifact behavior.

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing Journey J5 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] 3.1 `openspec validate add-journey-j5-patch-overlay --strict`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
