## 1. Implementation

- [x] Add `tests/journeys/common` with a `TestEnv` helper for journey tests.
- [x] Add a smoke test that uses the harness to run `agentpack init` in a temp environment.

## 2. Spec deltas

- [x] Add a delta requirement describing the harness (archive with `--skip-specs` since this is tests-only).

## 3. Validation

- [x] `openspec validate add-journey-harness --strict`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
