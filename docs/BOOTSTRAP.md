# Bootstrap (AI operator assets)

> Language: English | [Chinese (Simplified)](zh-CN/BOOTSTRAP.md)

Bootstrap’s goal is to make “how to use agentpack” self-serve for agents.

After running bootstrap once:
- Codex gains an `agentpack-operator` skill that teaches how to call the `agentpack` CLI (preferring `--json`) and the recommended workflows.
- Claude Code gains a set of `/ap-*` slash commands that wrap common operations (`doctor/update/preview/plan/diff/deploy/status/explain/evolve`) with minimal `allowed-tools`.
- Optionally (when enabled via `targets.claude_code.options.write_*_skills`), Claude Code gains an `agentpack-operator` Skill that teaches when to use Agentpack and points to `/ap-*` for execution.

## 1) Command

`agentpack bootstrap [--scope user|project|both]`

- `--scope` defaults to `both`: writes to both user and project locations
- Choose which targets to write via the global `--target`:
  - `agentpack --target codex bootstrap`
  - `agentpack --target claude_code bootstrap`

## 2) Output locations

- Codex:
  - user: `~/.codex/skills/agentpack-operator/SKILL.md`
  - project: `<project_root>/.codex/skills/agentpack-operator/SKILL.md`

- Claude Code:
  - user: `~/.claude/commands/ap-*.md`
  - project: `<project_root>/.claude/commands/ap-*.md`
  - user (optional): `~/.claude/skills/agentpack-operator/SKILL.md`
  - project (optional): `<project_root>/.claude/skills/agentpack-operator/SKILL.md`

These files are also included in the per-root target manifest (`.agentpack.manifest.json`), which means:
- `status` can detect them
- `rollback` can revert them
- deletes remove managed files only

## 3) Version marker and updates

Bootstrap templates replace `{{AGENTPACK_VERSION}}` with the current agentpack version.

After upgrading agentpack, if `status` reports operator assets are outdated:
- Re-run `agentpack bootstrap` to update them.

## 4) dry-run and `--json`

- Preview (no writes): `agentpack bootstrap --dry-run --json`
- Apply (writes):
  - human: `agentpack bootstrap` (interactive confirmation)
  - json: `agentpack --json bootstrap --yes`

Note: bootstrap is mutating; in `--json` mode you must pass `--yes` or the command fails with `E_CONFIRM_REQUIRED`.

## 5) Custom operator assets (optional)

Bootstrap uses built-in templates (updated with releases):
- `templates/codex/skills/agentpack-operator/SKILL.md`
- `templates/claude/commands/ap-*.md`
- `templates/claude/skills/agentpack-operator/SKILL.md`

If you want full customization:
- Package your own operator assets as normal modules (`skill`/`command`) managed via the manifest, or
- Use overlays after bootstrap to override the written files (recommended if you want to store “your own variant” in the config repo).

## 6) Claude Code `allowed-tools`

Claude Code slash commands shipped by bootstrap include YAML frontmatter `allowed-tools` to restrict tool access.

Design principles:
- Prefer the minimal set of `Bash("...")` entries required by the command.
- Keep commands single-purpose (e.g. `/ap-plan` only allows `agentpack plan --json`).
- Mutating commands should include explicit approval semantics (`--yes --json`) and be guarded (see next section).

## 7) Claude Code mutating command safety

Mutating Claude Code operator commands (e.g., `/ap-update`, `/ap-deploy`, `/ap-evolve`) are shipped with:
- `disable-model-invocation: true`

This reduces accidental programmatic invocation by the model while keeping explicit user-invoked actions available. Agentpack still enforces `--yes` for mutating operations in `--json` mode as the final guardrail.
