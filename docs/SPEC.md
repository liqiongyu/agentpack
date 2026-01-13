# Spec (implementation contract)

> Current as of **v0.5.0** (2026-01-13). This is the project’s **single authoritative spec**, aligned to the current implementation. Historical iterations live in git history; the repo no longer keeps `docs/versions/` snapshots.

## 0. Conventions

Command name: `agentpack`

Config repo: the agentpack config repo (a local clone), by default at `$AGENTPACK_HOME/repo`.

Default data directory: `~/.agentpack` (override via `AGENTPACK_HOME`), with:
- `repo/` (config repo, git; contains `agentpack.yaml` and `agentpack.lock.json`)
- `cache/` (git sources cache)
- `state/snapshots/` (deploy/rollback snapshots)
- `state/logs/` (record events)

Optional durability mode: set `AGENTPACK_FSYNC=1` to request `fsync` on atomic writes (slower, but more crash-consistent).

Supported as of v0.5.0:
- targets: `codex`, `claude_code`, `cursor`
- module types: `instructions`, `skill`, `prompt`, `command`
- source types: `local_path`, `git` (`url` + `ref` + `subdir`)

All commands default to human-readable output; pass `--json` for machine-readable JSON (envelope includes `schema_version`, `warnings`, and `errors`).

### 0.1 Stable error codes in `--json` mode (external contract)

When `--json` is enabled, common actionable failures must return stable error codes in `errors[0].code`:
- `E_CONFIRM_REQUIRED`: in `--json` mode, a mutating command is missing `--yes`.
- `E_ADOPT_CONFIRM_REQUIRED`: would overwrite an existing unmanaged file (`adopt_update`), but `--adopt` was not provided.
- `E_CONFIG_MISSING`: missing `repo/agentpack.yaml`.
- `E_CONFIG_INVALID`: `agentpack.yaml` is syntactically or semantically invalid (e.g. missing default profile, duplicate module id, invalid source config).
- `E_CONFIG_UNSUPPORTED_VERSION`: `agentpack.yaml` `version` is unsupported.
- `E_LOCKFILE_MISSING`: missing `repo/agentpack.lock.json` but the command requires it (e.g. `fetch`).
- `E_LOCKFILE_INVALID`: `agentpack.lock.json` is invalid JSON.
- `E_LOCKFILE_UNSUPPORTED_VERSION`: `agentpack.lock.json` `version` is unsupported.
- `E_TARGET_UNSUPPORTED`: an unsupported target (manifest targets or CLI `--target` selection).
- `E_DESIRED_STATE_CONFLICT`: multiple modules produced different content for the same `(target, path)` (refuse silent overwrite).
- `E_OVERLAY_NOT_FOUND`: overlay directory does not exist (overlay not created yet).
- `E_OVERLAY_BASELINE_MISSING`: overlay baseline metadata is missing (cannot rebase safely).
- `E_OVERLAY_BASELINE_UNSUPPORTED`: baseline has no locatable merge base (cannot rebase safely).
- `E_OVERLAY_REBASE_CONFLICT`: overlay rebase produced conflicts requiring manual resolution.

See: `ERROR_CODES.md`.

Note: In `--json` mode, unclassified/unexpected failures use the non-stable fallback code `E_UNEXPECTED` (see: `JSON_API.md` and `ERROR_CODES.md`).

## 1. Core concepts and data model

### 1.1 Module

Logical fields:
- `id: string` (globally unique; recommended `type/name`)
- `type: oneof [instructions, skill, prompt, command]`
- `source: Source`
- `enabled: bool` (default `true`)
- `tags: [string]` (used by profiles)
- `targets: [string]` (restrict to specific targets; default all)
- `metadata`:
  - `name` / `description` (optional)

### 1.2 Source

- `local_path`:
  - `path: string` (repo-relative path or absolute path)
- `git`:
  - `url: string`
  - `ref: string` (tag/branch/commit; default `main`)
  - `subdir: string` (path within repo; optional)
  - `shallow: bool` (default `true`)

### 1.3 Profile

- `name: string`
- `include_tags: [string]`
- `include_modules: [module_id]`
- `exclude_modules: [module_id]`

### 1.4 Target

- `name: oneof [codex, claude_code, cursor]`
- `mode: oneof [files]` (v0.1)
- `scope: oneof [user, project, both]`
- `options: map` (target-specific)

### 1.5 Project identity (for project overlays)

`project_id` generation rules (priority order):
1) hash of the normalized git remote `origin` URL (recommended)
2) if no remote: hash of the repo root absolute path

`project_id` must be stable (same project across machines).

## 2. Config files

### 2.1 `repo/agentpack.yaml` (manifest)

Example:

```yaml
version: 1

profiles:
  default:
    include_tags: ["base"]
  work:
    include_tags: ["base", "work"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"           # can be overridden by CODEX_HOME
      write_repo_skills: true          # write to $REPO_ROOT/.codex/skills
      write_user_skills: true          # write to ~/.codex/skills
      write_user_prompts: true         # write to ~/.codex/prompts
      write_agents_global: true        # write to ~/.codex/AGENTS.md
      write_agents_repo_root: true     # write to <repo>/AGENTS.md
  claude_code:
    mode: files
    scope: both
    options:
      write_repo_commands: true        # write to <repo>/.claude/commands
      write_user_commands: true        # write to ~/.claude/commands
      write_repo_skills: false         # optional in v0.1 (can keep off)
      write_user_skills: false

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"

  - id: skill:git-review
    type: skill
    tags: ["work"]
    source:
      git:
        url: "https://github.com/your-org/agentpack-modules.git"
        ref: "v1.2.0"
        subdir: "skills/git-review"

  - id: prompt:draftpr
    type: prompt
    tags: ["work"]
    source:
      local_path:
        path: "modules/prompts/draftpr.md"

  - id: command:ap-plan
    type: command
    tags: ["base"]
    source:
      local_path:
        path: "modules/claude-commands/ap-plan.md"
```

Notes:
- `instructions` module sources point to a directory, which may contain:
  - `AGENTS.md` (template)
  - rule fragments (future extension)
- `skill` module sources point to the skill directory root (contains `SKILL.md`)
- `prompt` module sources point to a single `.md` file (Codex custom prompt)
- `command` module sources point to a single Claude slash command `.md` file

### 2.2 `repo/agentpack.lock.json` (lockfile)

Minimal fields:
- `version: 1`
- `generated_at: ISO8601`
- `modules: [ { id, type, resolved_source, resolved_version, sha256, file_manifest } ]`

Where:
- `resolved_source: { ... }`
- `resolved_version: string` (commit sha or semver tag)
- `file_manifest: [{path, sha256, bytes}]`

Requirements:
- The lockfile must be diff-friendly (stable JSON key order; stable array ordering).
- `fetch` can only use lockfile `resolved_version` values.
- For `local_path` modules: `resolved_source.local_path.path` must be stored as a repo-relative path (never absolute), and must use `/` separators to keep cross-machine diffs stable.

### 2.3 `<target root>/.agentpack.manifest.json` (target manifest)

Goals:
- Safe delete (delete managed files only)
- Drift/status (`modified` / `missing` / `extra`)

Schema (v1 example):

```json
{
  "schema_version": 1,
  "generated_at": "2026-01-11T00:00:00Z",
  "tool": "codex",
  "snapshot_id": "optional",
  "managed_files": [
    {
      "path": "skills/agentpack-operator/SKILL.md",
      "sha256": "…",
      "module_ids": ["skill:agentpack-operator"]
    }
  ]
}
```

Requirements:
- `path` must be a relative path and must not contain `..`.
- The manifest records only files written by agentpack deployments; never treat user-native files as managed files.
- Readers MUST tolerate unsupported `schema_version` by emitting a warning and treating the manifest as missing (fall back behavior).

### 2.4 `state/logs/events.jsonl` (event log)

The event log written by `agentpack record` is JSON Lines (one JSON object per line).

Line shape (v1 example):

```json
{
  "schema_version": 1,
  "recorded_at": "2026-01-11T00:00:00Z",
  "machine_id": "my-macbook",
  "module_id": "command:ap-plan",
  "success": true,
  "event": { "module_id": "command:ap-plan", "success": true }
}
```

Conventions:
- `event` is arbitrary JSON; `score` only parses `module_id|moduleId` and `success|ok`.
- Top-level `module_id` and `success` are optional (compat with historical logs); `score` prefers them if present.
- `score` must tolerate bad lines (truncated / invalid JSON): skip with a warning rather than failing the entire command.
- Compatibility:
  - Adding new top-level fields is allowed (old readers ignore unknown fields).
  - If a line has an unsupported `schema_version`: skip with a warning (do not abort the whole command).
  - `score --json` includes skipped line counts and reason stats in `data.read_stats` to help diagnose log health.
- Optional top-level fields (additive, v1): `command_id`, `duration_ms`, `git_rev`, `snapshot_id`, `targets`.

## 3. Overlays

### 3.1 Overlay layers and precedence

Final composition order (low → high):
1) upstream module (local repo dir or cached checkout)
2) global overlay (`repo/overlays/<module_fs_key>/...`)
3) machine overlay (`repo/overlays/machines/<machine_id>/<module_fs_key>/...`)
4) project overlay (`repo/projects/<project_id>/overlays/<module_fs_key>/...`)

Where:
- `module_fs_key` is a cross-platform-safe directory name derived from `module_id` (sanitized, plus a short hash to avoid collisions).
- The CLI and manifests use the original `module_id`; `module_fs_key` is only for disk addressing.

### 3.2 Overlay representation (v0.2)

Overlay uses a “file override” model:
- overlay directory structure mirrors the upstream module
- same-path files override upstream
- (future) patch/diff overlays (e.g. 3-way merge based) are possible but not implemented yet

### 3.3 Overlay editing commands (see CLI)

`agentpack overlay edit <module_id> [--scope global|machine|project] [--sparse|--materialize]`:
- if the overlay does not exist: by default it copies the entire upstream module tree into the overlay directory (scope path mapping below)
- opens the editor (`$EDITOR`)
- after saving: changes take effect via deploy

Implemented options:
- `--sparse`: create a sparse overlay (write metadata only; do not copy upstream files; users add only changed files).
- `--materialize`: “fill in” missing upstream files into the overlay directory (copy missing files only; never overwrite existing overlay edits).

`agentpack overlay rebase <module_id> [--scope global|machine|project] [--sparsify]`:
- reads `<overlay_dir>/.agentpack/baseline.json` as merge base
- performs 3-way merge for files modified in the overlay (merge upstream updates into overlay edits)
- for files that were copied into overlay but not modified (`ours == base`): update them to latest upstream (avoid unintentionally pinning old versions)
- on success: refresh baseline (so drift warnings are computed from the latest upstream)
- on conflicts: overlay files contain conflict markers; in `--json` mode return stable error code `E_OVERLAY_REBASE_CONFLICT` (details include the conflict file list)

Optional:
- `--sparsify`: delete overlay files that are identical to upstream after rebase (keep overlays minimal).

Scope → path mapping:
- global: `repo/overlays/<module_fs_key>/...`
- machine: `repo/overlays/machines/<machine_id>/<module_fs_key>/...`
- project: `repo/projects/<project_id>/overlays/<module_fs_key>/...`

Compatibility:
- `--project` is still accepted but deprecated (equivalent to `--scope project`).

Additional (v0.3+):
- `agentpack overlay path <module_id> [--scope global|machine|project]`
  - human: prints absolute overlay dir path
  - json: returns `data.overlay_dir`

### 3.4 Overlay metadata (`.agentpack/`)

- Overlay skeleton writes `<overlay_dir>/.agentpack/baseline.json` for overlay drift warnings (not deployed).
- `.agentpack/` is a reserved metadata directory: it is never deployed to target roots and must not appear in module outputs.

## 4. CLI commands (v0.5.0)

Global flags:
- `--repo <path>`: config repo location
- `--profile <name>`: default `default`
- `--target <name|all>`: default `all`
- `--machine <id>`: machine overlay id (default: auto-detected machineId)
- `--json`: JSON output
- `--yes`: skip confirmation prompts
- `--dry-run`: force no writes (even for `deploy --apply`); default false

Safety guardrails:
- In `--json` mode, commands that write to disk and/or mutate git require `--yes` (avoid accidental writes in scripts/LLMs).
- If `--yes` is missing: exit code is non-zero, stdout is still valid JSON (`ok=false`), and a stable error code `E_CONFIRM_REQUIRED` is returned in `errors[0].code`.

### 4.1 `init`

`agentpack init [--git] [--bootstrap]`
- creates `$AGENTPACK_HOME/repo` (use `--git` to also run `git init` and write/update a minimal `.gitignore`)
- writes a minimal `agentpack.yaml` skeleton
- creates a `modules/` directory

Optional:
- `--git`: ensure `.gitignore` contains `.agentpack.manifest.json` (idempotent).
- `--bootstrap`: install operator assets into the config repo after init (equivalent to `agentpack bootstrap --scope project`).

### 4.2 `add` / `remove`

- `agentpack add <type> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code,cursor]`
- `agentpack remove <module_id>`

Source expressions:
- `local:modules/xxx`
- `git:https://...#ref=...&subdir=...`

### 4.3 `lock`

`agentpack lock`
- resolves all module sources
- generates/updates the lockfile

### 4.4 `fetch` (install)

`agentpack fetch`
- materializes lockfile modules into the cache (git sources checkout)
- validates sha256

v0.3+ behavior hardening (fewer footguns):
- when the lockfile exists but a `<moduleId, commit>` checkout cache is missing, `plan/diff/deploy/overlay edit` will auto-fetch the missing checkout (a safe network operation), rather than forcing users to run `fetch` manually first.

### 4.4.1 `update` (composite)

`agentpack update [--lock] [--fetch] [--no-lock] [--no-fetch]`
- default strategy:
  - if lockfile does not exist: run `lock` + `fetch`
  - if lockfile exists: run `fetch` only by default
- purpose: reduce friction in the common lock/fetch workflow, especially for AI/script orchestration.

Notes:
- In `--json` mode, `update` is treated as mutating and requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- `--json` output aggregates steps: `data.steps=[{name, ok, detail}, ...]`.

### 4.4.2 `preview` (composite)

`agentpack preview [--diff]`
- always runs `plan`
- when `--diff` is set: also computes and prints diff (human: unified diff; json: diff summary)

Notes:
- `preview` is read-only and does not require `--yes`.

### 4.5 `plan` / `diff`

`agentpack plan`
- shows which targets/files would be written and what operation would be performed (`create` / `update` / `delete`)
- if multiple modules produce the same `(target, path)`:
  - same content: merge `module_ids` (for provenance/explain)
  - different content: error and return `E_DESIRED_STATE_CONFLICT` (block apply by default)

`agentpack diff`
- prints per-file text diffs; in JSON mode prints diff summary + file hash changes
- for `update` operations: JSON includes `update_kind` (`managed_update` / `adopt_update`)

### 4.6 `deploy`

`agentpack deploy [--apply] [--adopt]`

Default behavior:
- runs `plan`
- shows diff
- when `--apply` is set:
  - performs apply (with backup) and writes a state snapshot
  - writes `.agentpack.manifest.json` under each target root
- delete protection: only deletes managed files recorded in the manifest (never deletes unmanaged user files)
- overwrite protection: refuses to overwrite existing unmanaged files (`adopt_update`) unless `--adopt` is provided
- without `--apply`: show plan only (equivalent to `plan` + `diff`)

Notes:
- `--json` + `--apply` requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- If the plan contains any `adopt_update`, apply requires `--adopt`; in `--json` mode, missing `--adopt` returns `E_ADOPT_CONFIRM_REQUIRED`.
- Even if the plan is empty, if the target root is missing a manifest, agentpack writes a manifest (so drift/safe-delete works going forward).

### 4.7 `status`

`agentpack status`
- if the target root contains `.agentpack.manifest.json`: compute drift (`modified` / `missing` / `extra`) based on the manifest
- if there is no manifest (or the manifest has an unsupported `schema_version`): fall back to comparing desired outputs vs filesystem, and emit a warning
- if installed operator assets (bootstrap) are missing or outdated: emit a warning and suggest running `agentpack bootstrap`

### 4.8 `rollback`

`agentpack rollback --to <snapshot_id>`
- restores backups
- records a rollback event

### 4.9 `bootstrap` (AI-first operator assets)

`agentpack bootstrap [--target codex|claude_code|all] [--scope user|project|both]`
- installs operator assets:
  - Codex: writes one skill (`agentpack-operator`)
  - Claude: writes a set of slash commands (`ap-doctor`, `ap-update`, `ap-preview`, `ap-plan`, `ap-diff`, `ap-deploy`, `ap-status`, `ap-explain`, `ap-evolve`)
- asset contents come from embedded templates shipped with agentpack (updated with versions)
- each operator file includes a version marker: `agentpack_version: x.y.z` (frontmatter or comment)

Requirement:
- If a Claude command uses bash execution, it must declare `allowed-tools` (minimal set).

Notes:
- In `--json` mode, `bootstrap` requires `--yes` (it writes to target roots; otherwise `E_CONFIRM_REQUIRED`).

### 4.10 `doctor`

`agentpack doctor [--fix]`
- prints machineId (used for machine overlays)
- checks target roots exist and are writable, with actionable suggestions (mkdir/permissions/config)
- git hygiene (v0.3+):
  - if a target root is inside a git repo and `.agentpack.manifest.json` is not ignored: emit a warning (avoid accidental commits)
  - `--fix`: idempotently appends `.agentpack.manifest.json` to that repo’s `.gitignore`
    - in `--json` mode, if it writes, it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`)

### 4.11 `remote` / `sync`

- `agentpack remote set <url> [--name origin]`
- `agentpack sync [--rebase] [--remote origin]`

Behavior:
- wraps a recommended multi-machine sync flow with git commands (`pull --rebase` + `push`)
- does not resolve conflicts automatically; on conflict it fails and asks the user to handle it

### 4.12 `record` / `score`

- `agentpack record` (reads JSON from stdin and appends to `state/logs/events.jsonl`)
- `agentpack score` (computes failure rates from `events.jsonl`)

Event conventions (v0.2):
- `record` treats stdin JSON as `event` (no strict schema).
- `score` identifies:
  - module id: `module_id` or `moduleId`
  - success: `success` or `ok` (default to true if missing)

### 4.13 `explain`

`agentpack explain plan|diff|status`
- prints “provenance explanation” for changes/drift: moduleId + overlay layer (`project` / `machine` / `global` / `upstream`)

### 4.14 `evolve propose`

`agentpack evolve propose [--module-id <id>] [--scope global|machine|project]`
- captures drifted deployed file contents and generates overlay changes (creates a proposal branch in the config repo; does not auto-deploy)

Notes:
- In `--json` mode it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- Requires a clean working tree in the config repo; it creates a branch and attempts to commit.
  - If git identity is missing and commit fails, agentpack prints guidance and keeps the branch and changes.
- Current behavior is conservative: only generate proposals for drift that can be safely attributed to a single module.
  - By default it only processes outputs with `module_ids.len() == 1`.
  - For aggregated Codex `AGENTS.md` (composed from multiple `instructions` modules): if the file contains segment markers, agentpack tries to map drift back to the corresponding module segment and propose changes.
    - If markers are missing/unparseable, it skips with a `multi_module_output` reason.
  - It only processes drift where the deployed file exists but content differs; it skips `missing` drift (recommend `deploy` to restore).
  - Recommended flow: run `agentpack evolve propose --dry-run --json` to inspect `candidates` / `skipped` / warnings, then decide whether to pass `--yes` to create the proposal branch.

Aggregated instructions marker format (implemented; example):

```md
<!-- agentpack:module=instructions:one -->
# one
<!-- /agentpack -->
```

### 4.15 `evolve restore`

`agentpack evolve restore [--module-id <id>]`
- restores `missing` desired outputs to disk in a “create-only” way (creates missing files only; does not update existing files; does not delete anything)

Notes:
- In `--json` mode, if it writes, it requires `--yes` (otherwise `E_CONFIRM_REQUIRED`).
- Supports `--dry-run`: prints the file list only; does not write.

### 4.16 `help` / `schema` (utility commands)

`agentpack help`
- prints CLI help/usage
- `agentpack help --json` emits machine-consumable command metadata (see: `JSON_API.md`), including at minimum:
  - `data.commands[]` (command catalog)
  - `data.mutating_commands[]` (command IDs that require `--yes` in `--json` mode)
  - `data.global_args[]` (global flags)

`agentpack schema`
- prints a brief JSON schema summary (human mode)
- `agentpack schema --json` documents:
  - `data.envelope` (the `schema_version=1` envelope fields/types)
  - `data.commands` (minimum expected `data` fields for key read commands)

## 5. Target adapter details

### 5.1 `codex` target

Paths (follow Codex docs):
- `codex_home`: `~/.codex` (override via `CODEX_HOME`)
- user skills: `$CODEX_HOME/skills`
- repo skills: per Codex skill precedence:
  - `$CWD/.codex/skills`
  - `$CWD/../.codex/skills`
  - `$REPO_ROOT/.codex/skills`
- custom prompts: `$CODEX_HOME/prompts` (user scope only)
- global agents: `$CODEX_HOME/AGENTS.md`
- repo agents: `<repo>/AGENTS.md`

Deploy rules:
- skills: copy directories (no symlinks)
- prompts: copy `.md` files into the prompts directory
- instructions:
  - global: render base `AGENTS.md` into `$CODEX_HOME/AGENTS.md`
  - project: render into repo-root `AGENTS.md` (default)
  - (future) finer-grained subdir override

### 5.2 `claude_code` target (files mode)

Paths:
- repo commands: `<repo>/.claude/commands`
- user commands: `~/.claude/commands`

Deploy rules:
- command modules are single `.md` files; filename = slash command name
- if the body uses `!bash`/`!`bash``: the YAML frontmatter must declare `allowed-tools: Bash(...)`
- (future) plugin mode is possible (write `.claude-plugin/plugin.json`), but not implemented yet

### 5.3 `cursor` target (files mode)

Paths:
- project rules: `<project_root>/.cursor/rules` (project scope only)

Deploy rules:
- instructions:
  - for each enabled `instructions` module, write one Cursor rule file:
    - `<project_root>/.cursor/rules/<module_fs_key>.mdc`
  - each rule file includes YAML frontmatter (`description`, `globs`, `alwaysApply`) and the module’s `AGENTS.md` content.

Notes:
- `cursor` currently supports project scope only; `scope: user` is invalid.

## 6. JSON output spec

See: `JSON_API.md`.

All `--json` outputs must include:
- `schema_version: number`
- `ok: boolean`
- `command: string`
- `version: string` (agentpack version)
- `data: object` (empty object on failure)
- `warnings: [string]`
- `errors: [{code, message, details?}]`

Path field convention:
- Whenever a JSON payload contains filesystem paths (e.g. `path`, `root`, `repo`, `overlay_dir`, `lockfile`, ...), it should also provide a companion `*_posix` field using `/` separators.
- This is additive (no `schema_version` bump): original fields remain unchanged; automation should prefer parsing `*_posix` for cross-platform stability.

`plan --json` `data` example:

```json
{
  "profile": "work",
  "targets": ["codex", "claude_code"],
  "changes": [
    {
      "target": "codex",
      "op": "update",
      "path": "/home/user/.codex/skills/agentpack-operator/SKILL.md",
      "path_posix": "/home/user/.codex/skills/agentpack-operator/SKILL.md",
      "before_sha256": "...",
      "after_sha256": "...",
      "update_kind": "managed_update",
      "reason": "content differs"
    }
  ],
  "summary": {"create": 3, "update": 2, "delete": 0}
}
```

`status --json` `data` example:

```json
{
  "drift": [
    {
      "target": "codex",
      "path": "...",
      "path_posix": "...",
      "expected": "sha256:...",
      "actual": "sha256:...",
      "kind": "modified"
    }
  ]
}
```

## 7. Compatibility and limitations

- No symlinks by default (unless a future experimental `--link` flag is added).
- Do not execute third-party scripts.
- Prompts do not support repo scope (follow Codex docs); use a skill to share prompts.

## 8. References

(Same as `PRD.md`.)
