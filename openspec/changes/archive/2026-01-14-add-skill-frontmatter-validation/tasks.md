## 1. Spec & docs
- [x] Update `docs/SPEC.md` (bootstrap/operator assets and skill module expectations) to document required `SKILL.md` frontmatter fields.
- [x] Update `docs/ERROR_CODES.md` to document `E_CONFIG_INVALID` for invalid skill frontmatter (additive clarification).

## 2. Implementation
- [x] Validate `skill` modules’ `SKILL.md` YAML frontmatter and required fields (`name`, `description`).
- [x] Surface invalid skill frontmatter as `E_CONFIG_INVALID` in `--json` mode with actionable details.

## 3. Tests
- [x] Add integration tests covering valid and invalid `SKILL.md` frontmatter.

## 4. Validation
- [x] `openspec validate add-skill-frontmatter-validation --strict --no-interactive`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
