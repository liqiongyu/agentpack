# Targets (`codex` / `claude_code`)

> Language: English | [Chinese (Simplified)](zh-CN/TARGETS.md)

Targets define where agentpack writes the “compiled assets”, and which directories must write `.agentpack.manifest.json` to enable safe deletes and drift detection.

Built-in targets:
- `codex`
- `claude_code`

For shared target fields, see `CONFIG.md`.

## 1) codex

### Managed roots

Determined by `targets.codex.scope` and `targets.codex.options.*`, and may include:
- `~/.codex` (global instructions: `AGENTS.md`)
- `~/.codex/skills` (user skills)
- `~/.codex/prompts` (custom prompts; user scope only)
- `<project_root>/AGENTS.md` (project instructions)
- `<project_root>/.codex/skills` (repo skills)

Notes:
- `project_root` is derived from the current working directory’s project identity (usually the git repo root).
- Each root writes (or updates) `.agentpack.manifest.json` for safe deletes and drift detection.

### Module → output mapping

- `instructions`
  - Collects each instructions module’s `AGENTS.md`
  - When multiple modules exist, agentpack generates a single `AGENTS.md` with per-module section markers to support `evolve propose` mapping for aggregated files

- `skill`
  - Copies all files under the module directory to:
    - `~/.codex/skills/<skill_name>/...` (if user skills are enabled)
    - `<project_root>/.codex/skills/<skill_name>/...` (if repo skills are enabled)
  - `<skill_name>` is derived from the module id (`skill:<name>`) when possible; otherwise it is sanitized

- `prompt`
  - Copies a single `.md` file to `~/.codex/prompts/<filename>.md`

### Common options

- `codex_home`: default `"~/.codex"`
- `write_repo_skills`: default true (requires project scope)
- `write_user_skills`: default true (requires user scope)
- `write_user_prompts`: default true (requires user scope)
- `write_agents_global`: default true (requires user scope)
- `write_agents_repo_root`: default true (requires project scope)

### Limitations and tips

- Agentpack uses copy/render by default (no symlinks) to keep discovery reliable in Codex.
- Prompts are written to user scope only (`~/.codex/prompts`) per Codex semantics. If you want to share reusable behavior, prefer a skill instead.

## 2) claude_code

### Managed roots

- `~/.claude/commands` (user commands; enabled by default)
- `<project_root>/.claude/commands` (repo commands; enabled by default)

### Module → output mapping

- `command`
  - Copies a single `.md` file into the commands directory
  - The filename becomes the slash command name (e.g. `ap-plan.md` → `/ap-plan`)

### Frontmatter requirements (important)

Claude Code custom command files require YAML frontmatter.

Minimal example:

```md
---
description: Plan changes with agentpack
allowed-tools:
  - Bash(agentpack*)
  - Bash(git status)
---

# /ap-plan
...
```

Rules:
- `description` is required
- If the body contains `!bash` or `!`bash``: you must declare `allowed-tools` and include `Bash(...)`

## 3) scan_extras (handling extra files)

Some roots enable `scan_extras`:
- `true`: `status` reports “extra” files that exist on disk but are not in the managed manifest (never auto-deleted)
- `false`: do not scan extras (e.g., the global `~/.codex` root typically avoids full scans)

## 4) Adding a new target?

See:
- `TARGET_SDK.md`
- `TARGET_CONFORMANCE.md`
