## 1. Implementation

- [x] 1.1 Add `just nextest` recipe.
- [x] 1.2 Add CI job that installs and runs `cargo nextest run` (optional / non-blocking).

## 2. Spec deltas

- [x] 2.1 Add a delta requirement describing the CI nextest path (archive with `--skip-specs` since this is tooling-only).

## 3. Validation

- [x] 3.1 `openspec validate add-ci-nextest --strict`
- [x] 3.2 `cargo fmt --all -- --check`
- [x] 3.3 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 `cargo test --all --locked`
