# Targets (`codex` / `claude_code` / `cursor` / `vscode` / `jetbrains` / `zed`)

> Language: English | [Chinese (Simplified)](../zh-CN/reference/targets.md)

Targets define where agentpack writes the “compiled assets”, and which directories must write `.agentpack.manifest.<target>.json` to enable safe deletes and drift detection.

Built-in targets:
- `codex`
- `claude_code`
- `cursor`
- `vscode`
- `jetbrains`
- `zed`
- `export_dir` (experimental, feature-gated)

## Capability matrix (at a glance)

| Target | Maturity | Scopes | Module types | Key outputs (typical) |
| --- | --- | --- | --- | --- |
| `codex` | stable | user / project / both | `instructions`, `skill`, `prompt` | `~/.codex/AGENTS.md`<br>`~/.codex/skills/<name>/...`<br>`~/.codex/prompts/<file>.md`<br>`<project_root>/AGENTS.md`<br>`<project_root>/.codex/skills/<name>/...` |
| `claude_code` | stable | user / project / both | `command`, `skill` | `~/.claude/commands/<name>.md`<br>`<project_root>/.claude/commands/<name>.md`<br>`~/.claude/skills/<name>/...` (opt-in)<br>`<project_root>/.claude/skills/<name>/...` (opt-in) |
| `cursor` | stable | project | `instructions` | `<project_root>/.cursor/rules/<module>.mdc` |
| `vscode` | stable | project | `instructions`, `prompt` | `<project_root>/.github/copilot-instructions.md`<br>`<project_root>/.github/prompts/<name>.prompt.md` |
| `jetbrains` | stable | project | `instructions` | `<project_root>/.junie/guidelines.md` |
| `zed` | stable | project | `instructions` | `<project_root>/.rules` |
| `export_dir` | experimental | user / project / both | `instructions`, `skill`, `prompt`, `command` | `<export_root>/AGENTS.md`<br>`<export_root>/skills/<name>/...`<br>`<export_root>/prompts/<file>.md`<br>`<export_root>/commands/<file>.md` |

Notes:
- Exact roots/paths can vary based on target options (especially `codex`); see the per-target sections below.
- `cursor`, `vscode`, `jetbrains`, and `zed` are project-scope targets.
- `export_dir` is feature-gated (Cargo feature: `target-export-dir`). When `scope: both`, outputs are written under `<export_root>/user/` and `<export_root>/project/`.

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
- Each root writes (or updates) `.agentpack.manifest.<target>.json` for safe deletes and drift detection.

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
- `~/.claude/skills` (user skills; disabled by default)
- `<project_root>/.claude/skills` (repo skills; disabled by default)

### Module → output mapping

- `command`
  - Copies a single `.md` file into the commands directory
  - The filename becomes the slash command name (e.g. `ap-plan.md` → `/ap-plan`)

- `skill`
  - Copies all files under the module directory to:
    - `~/.claude/skills/<skill_name>/...` (if user skills are enabled)
    - `<project_root>/.claude/skills/<skill_name>/...` (if repo skills are enabled)
  - `<skill_name>` is derived from the module id (`skill:<name>`) when possible; otherwise it is sanitized

### Common options

- `write_repo_commands`: default true (requires project scope)
- `write_user_commands`: default true (requires user scope)
- `write_repo_skills`: default false (requires project scope)
- `write_user_skills`: default false (requires user scope)

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

## 3) cursor

Cursor rules are stored under `.cursor/rules` and use `.mdc` files with YAML frontmatter.

### Managed roots

- `<project_root>/.cursor/rules` (project scope only)

### Module → output mapping

- `instructions`
  - Writes one Cursor rule file per module:
    - `<project_root>/.cursor/rules/<module_fs_key>.mdc`
  - Frontmatter defaults:
    - `description: "agentpack: <module_id>"`
    - `globs: []`
    - `alwaysApply: true`

### Common options

- `write_rules`: default true (requires project scope)

Notes:
- `cursor` currently supports project scope only (`scope: user` is invalid).

## 4) vscode

VS Code / GitHub Copilot uses repository-scoped “custom instructions” and “prompt files” under `.github/`.

### Managed roots

- `<project_root>/.github` (instructions; `scan_extras=false` to avoid flagging unrelated `.github/*` files)
- `<project_root>/.github/prompts` (prompt files; `scan_extras=true`)

### Module → output mapping

- `instructions`
  - Collects each instructions module’s `AGENTS.md` content into:
    - `<project_root>/.github/copilot-instructions.md`
  - When multiple modules exist, agentpack generates a single file with per-module section markers to preserve attribution.

- `prompt`
  - Copies a single `.md` file into:
    - `<project_root>/.github/prompts/<name>.prompt.md`
  - If the source filename does not already end with `.prompt.md`, agentpack appends `.prompt.md` for discovery.

### Common options

- `write_instructions`: default true (requires project scope)
- `write_prompts`: default true (requires project scope)

Notes:
- `vscode` currently supports project scope only (`scope: user` is invalid).

## 5) jetbrains

JetBrains Junie can load project guidelines from `.junie/guidelines.md` by default (and also supports open formats like `AGENTS.md`).

This target writes Junie’s default guidelines file location so JetBrains users get working project instructions without extra IDE setup.

### Managed roots

- `<project_root>/.junie` (project scope only; `scan_extras=true`)

### Module → output mapping

- `instructions`
  - Collects each instructions module’s `AGENTS.md` content into:
    - `<project_root>/.junie/guidelines.md`
  - When multiple modules exist, agentpack generates a single file with per-module section markers to preserve attribution.

### Common options

- `write_guidelines`: default true (requires project scope)

Notes:
- `jetbrains` currently supports project scope only (`scope: user` is invalid).
- If you also use GitHub Copilot in JetBrains, the `vscode` target’s `.github/copilot-instructions.md` output may still be relevant (depending on your client’s support).

## 6) scan_extras (handling extra files)

Some roots enable `scan_extras`:
- `true`: `status` reports “extra” files that exist on disk but are not in the managed manifest (never auto-deleted)
- `false`: do not scan extras (e.g., the global `~/.codex` root typically avoids full scans)

## 7) Adding a new target?

See:
- `TARGET_MAPPING_TEMPLATE.md`
- `TARGET_SDK.md`
- `TARGET_CONFORMANCE.md`

## 8) zed

Zed supports repo-root `.rules` files for AI “rules”, and also supports other compatible filenames (see: https://zed.dev/docs/ai/rules.html).

### Managed roots

- `<project_root>` (project scope only; `scan_extras=false`)

### Module → output mapping

- `instructions`
  - Collects each instructions module’s `AGENTS.md` content into:
    - `<project_root>/.rules`

### Common options

- `write_rules`: default true (requires project scope)

Notes:
- `.rules` takes precedence over other compatible rule filenames in Zed’s search order.
- If you prefer not to create `.rules`, you can still use existing outputs like:
  - `vscode` → `.github/copilot-instructions.md`
  - `codex` (project) → `<project_root>/AGENTS.md`

Example (minimal) snippet:

```yaml
targets:
  zed:
    mode: files
    scope: project
    options:
      write_rules: true
```

## 9) export_dir (experimental)

This target is feature-gated (Cargo feature: `target-export-dir`) and exports compiled assets to a deterministic filesystem tree under `targets.export_dir.options.root`.

See `../targets/export_dir.md`.
