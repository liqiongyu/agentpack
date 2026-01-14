## 1. Spec
- [x] Update `docs/SPEC.md` to describe Claude Code skills paths and bootstrap behavior.
- [x] Update `docs/TARGETS.md` and `docs/BOOTSTRAP.md` (EN + zh-CN) with Claude Code skills mapping and config gating.

## 2. Templates
- [x] Add `templates/claude/skills/agentpack-operator/SKILL.md`.

## 3. Implementation
- [x] Implement `claude_code` target support for `skill` modules (write to `.claude/skills` when enabled).
- [x] Implement `agentpack bootstrap` optional install of the Claude operator Skill (gated by `write_*_skills`).
- [x] Implement `status` operator-asset checks for the Claude operator Skill (gated by `write_*_skills`).

## 4. Tests
- [x] Add integration tests validating bootstrap writes the Claude Skill when enabled.
- [x] Add integration tests validating `status` warns when the Claude Skill is missing/outdated when enabled.

## 5. Validation
- [x] `openspec validate add-claude-code-agent-skill --strict --no-interactive`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all --locked`
