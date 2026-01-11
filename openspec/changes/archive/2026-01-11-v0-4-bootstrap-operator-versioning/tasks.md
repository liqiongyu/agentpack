## 1. Specs
- [x] Add CLI delta requirements for operator version stamps and status warnings.

## 2. Implementation
- [x] Stamp Codex/Claude operator templates with `agentpack_version` at install time.
- [x] Add missing Claude operator command templates (doctor/update/preview/explain/evolve).
- [x] Make `agentpack status` warn on missing/outdated operator assets.

## 3. Tests
- [x] Add integration tests covering missing/outdated warnings for codex and claude_code.

## 4. Validation
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test`.
- [x] Run `openspec validate v0-4-bootstrap-operator-versioning --strict --no-interactive`.
