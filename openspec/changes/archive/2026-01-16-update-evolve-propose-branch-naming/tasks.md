## 1. Implementation

- [x] 1.1 Update `evolve propose` default branch naming to include scope/module/timestamp
- [x] 1.2 Update CLI help + docs to reflect new default branch name

## 2. Validation

- [x] 2.1 Run `openspec validate update-evolve-propose-branch-naming --strict`
- [x] 2.2 Run `cargo fmt --all -- --check`
- [x] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 2.4 Run `cargo test --all --locked`
