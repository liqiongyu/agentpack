## 1. Implementation

- [ ] 1.1 Extend `evolve propose` marker attribution beyond `AGENTS.md`
- [ ] 1.2 Add VS Code combined instructions test coverage
- [ ] 1.3 Update docs (`docs/SPEC.md`, `docs/EVOLVE.md`) to include VS Code

## 2. Validation

- [ ] 2.1 Run `openspec validate update-evolve-propose-marked-instructions-outputs --strict`
- [ ] 2.2 Run `cargo fmt --all -- --check`
- [ ] 2.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 2.4 Run `cargo test --all --locked`
