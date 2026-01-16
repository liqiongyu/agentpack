## 1. Implementation

- [ ] 1.1 Update `evolve propose` default branch naming to include scope/module/timestamp
- [ ] 1.2 Update CLI help + docs to reflect new default branch name

## 2. Validation

- [ ] 2.1 Run `openspec validate update-evolve-propose-branch-naming --strict`
- [ ] 2.2 Run `cargo fmt --all -- --check`
- [ ] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 2.4 Run `cargo test --all --locked`
