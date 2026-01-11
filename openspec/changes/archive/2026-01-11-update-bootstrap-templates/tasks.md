## 1. Implementation

### 1.1 Templates
- [x] Update `templates/codex/skills/agentpack-operator/SKILL.md` content to cover record/score/explain/evolve propose workflow.
- [x] Update relevant `templates/claude/commands/*.md` content to mention evolve usage.

### 1.2 Validation
- [x] Run `cargo test` to ensure no regressions (templates are embedded into the binary).
- [x] Run `openspec validate update-bootstrap-templates --strict`.
