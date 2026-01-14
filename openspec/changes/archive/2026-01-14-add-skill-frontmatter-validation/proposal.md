# Change: stricter SKILL.md frontmatter validation

## Why
Codex/Claude Skills are a key on-ramp for AI-first usage, but a malformed `SKILL.md` (missing or wrong YAML frontmatter) can look “deployed” while being unusable in the host tool.

We should detect these issues early and report them clearly, so users can fix modules quickly and automation can react reliably in `--json` mode.

## What Changes
- Validate `skill` modules’ `SKILL.md` YAML frontmatter during module materialization (plan/deploy/etc):
  - require YAML frontmatter (`--- ... ---`)
  - require `name` and `description` as non-empty strings
- In `--json` mode, return a stable error code when the skill frontmatter is invalid (use existing `E_CONFIG_INVALID`).
- Update docs/specs and tests to lock the behavior down.

## Impact
- Affected docs/specs: `docs/SPEC.md`, `docs/ERROR_CODES.md`, `openspec/specs/agentpack-cli/spec.md`.
- Affected code: `src/validate.rs` (skill module validation).
- Backward compatibility: changes behavior for invalid skill modules (now fails fast with a clear error).
