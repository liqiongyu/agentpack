## 1. Implementation

### 1.1 Golden snapshots for policy commands
- [x] Add a golden test for `policy lock` / `policy lint` JSON output.
- [x] Add `tests/golden/policy_lock_json_data.json`.
- [x] Add `tests/golden/policy_lint_json_data.json`.
- [x] Normalize temp paths for cross-OS stability.

### 1.2 Validation
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --all --locked`.

## 2. Spec deltas

- [x] Add a delta requirement describing the policy JSON golden snapshots (archive with `--skip-specs` since this is tests-only).
