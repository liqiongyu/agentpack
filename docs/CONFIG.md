# Config and modules (`agentpack.yaml`)

> Language: English | [Chinese (Simplified)](zh-CN/CONFIG.md)

Agentpack’s single source of truth is the config repo’s `agentpack.yaml`.

You can edit YAML by hand, but using `agentpack add/remove` is recommended (built-in validation; fewer footguns).

## File location

Default: `$AGENTPACK_HOME/repo/agentpack.yaml` (`AGENTPACK_HOME` defaults to `~/.agentpack`)

You can also set it via `agentpack --repo <path>`.

## Minimal example

```yaml
version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"
      write_repo_skills: true
      write_user_skills: true
      write_user_prompts: true
      write_agents_global: true
      write_agents_repo_root: true
  claude_code:
    mode: files
    scope: both
    options:
      write_repo_commands: true
      write_user_commands: true
      write_repo_skills: false
      write_user_skills: false

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"
```

## Full example repo

For a copy/pasteable working example (manifest + module files), see:
- `docs/examples/minimal_repo/`

## Field reference

### version

- Currently supported: `1`

### profiles

Profiles select which modules should be deployed for a run.

Fields:
- `include_tags: [string]`: include modules with these tags
- `include_modules: [module_id]`: explicitly include modules
- `exclude_modules: [module_id]`: explicitly exclude modules

Required:
- A `default` profile must exist.

### targets

Built-in targets:
- `codex`
- `claude_code`
- `cursor`

Per-target fields:
- `mode`: currently only `files`
- `scope`: `user|project|both`
- `options`: target-specific key/value (arbitrary YAML values)

Note:
- `scope` controls which roots are written (user dirs and/or project dirs).

### modules

Per-module fields:
- `id: string`: globally unique; recommended format is `type:name` (e.g. `skill:git-review`)
- `type: instructions|skill|prompt|command`
- `enabled: bool`: default true
- `tags: [string]`: used by profiles
- `targets: [string]`: restrict to specific targets; empty = all
- `source`: see below
- `metadata: {k: v}`: optional; passthrough for comments/annotations

#### source (two kinds)

1) local_path

```yaml
source:
  local_path:
    path: "modules/instructions/base"
```

Convention:
- Prefer repo-relative paths.

2) git

```yaml
	source:
	  git:
	    url: "https://github.com/your-org/agentpack-modules.git"
	    ref: "v1.2.0"      # tag/branch/commit; default is main
	    subdir: "skills/git-review"   # optional
	    shallow: true       # default true
	```

Notes:
- Git sources are locked to an exact commit (written to `agentpack.lock.json`) for reproducibility.

## Module type constraints (important)

Before rendering, Agentpack validates the materialized module structure:

- `instructions`: must contain `AGENTS.md`
- `skill`: must contain `SKILL.md`
- `prompt`: must contain exactly one `.md` file after materialization
- `command`: must contain exactly one `.md` file after materialization, and must include YAML frontmatter:
  - Required: `description`
  - If the body uses `!bash`/`!`bash``: frontmatter must include `allowed-tools` and allow `Bash(...)`

Tip: for prompt/command modules, the source can be a single file or a directory, but the materialized result must contain exactly one file.

See also:
- Target writing rules: `TARGETS.md`
- Overlay/source composition: `OVERLAYS.md`
