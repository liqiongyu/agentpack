## 1. Implementation

- [x] 1.1 Extend `evolve propose` marker attribution beyond `AGENTS.md`
- [x] 1.2 Add VS Code combined instructions test coverage
- [x] 1.3 Update docs (`docs/SPEC.md`, `docs/EVOLVE.md`) to include VS Code

## 2. Validation

- [x] 2.1 Run `openspec validate update-evolve-propose-marked-instructions-outputs --strict`
- [x] 2.2 Run `cargo fmt --all -- --check`
- [x] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 2.4 Run `cargo test --all --locked`
