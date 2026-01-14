# Change: Claude Code Agent Skill (optional complement to /ap-*)

## Why
Claude Code currently has explicit `/ap-*` command wrappers, but no Skill that helps Claude naturally discover “I should use agentpack now”.

This change adds an optional Claude Code Skill (default off) that teaches when to use Agentpack and points to `/ap-*` for execution, preserving safe-by-default behavior.

## What Changes
- Add a Claude Code Skill template at `templates/claude/skills/agentpack-operator/SKILL.md`.
- Extend the `claude_code` target to deploy `skill` modules into `.claude/skills` when enabled via `targets.claude_code.options.write_*_skills`.
- Extend `agentpack bootstrap` to optionally install the Claude operator Skill (gated by `write_*_skills`).
- Extend `agentpack status` operator-asset checks to include the Claude operator Skill when enabled.

## Impact
- Affected docs/specs: `docs/SPEC.md`, `docs/TARGETS.md`, `docs/BOOTSTRAP.md` (and zh-CN translations).
- Affected code: `src/engine.rs`, `src/cli/commands/bootstrap.rs`, `src/cli/commands/status.rs`.
- Backward compatibility: default behavior unchanged unless `write_*_skills` is enabled for `claude_code`.
