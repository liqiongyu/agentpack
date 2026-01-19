## 1. Implementation

- [x] Add Journey J3 integration test covering `adopt_update` refusal, then `--adopt` success, then follow-up `managed_update`.

## 2. Spec deltas

- [x] Add a delta requirement describing Journey J3 coverage (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] `openspec validate add-journey-j3-adopt-update --strict`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
