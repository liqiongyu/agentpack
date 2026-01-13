# CLI reference

> Language: English | [Chinese (Simplified)](zh-CN/CLI.md)

This document is for quickly looking up how a command works. For workflow-oriented guidance, see `WORKFLOWS.md`.

## Global flags (supported by all commands)

- `--repo <path>`: path to the config repo (default: `$AGENTPACK_HOME/repo`)
- `--profile <name>`: profile name (default: `default`)
- `--target <codex|claude_code|all>`: target selection (default: `all`)
- `--machine <id>`: override machine id (for machine overlays; default: auto-detect)
- `--json`: machine-readable JSON output (envelope) on stdout
- `--yes`: skip confirmations (note: in `--json` mode, mutating commands require explicit `--yes`)
- `--dry-run`: force dry-run behavior (even if `deploy --apply` / `overlay rebase` etc. are requested)

Tips:
- `agentpack help --json` returns a structured command list and the mutating command set.
- `agentpack schema --json` describes the JSON envelope and common `data` payload shapes.

## init

`agentpack init`
- Initializes a config repo skeleton (creates `agentpack.yaml` and example directories)
- Does not run `git init`

## add / remove

- `agentpack add <instructions|skill|prompt|command> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code]`
- `agentpack remove <module_id>`

Source spec:
- `local:<path>` (repo-relative path)
- `git:<url>#ref=<ref>&subdir=<path>`

Examples:
- `agentpack add instructions local:modules/instructions/base --id instructions:base --tags base`
- `agentpack add skill git:https://github.com/your-org/agentpack-modules.git#ref=v1.2.0&subdir=skills/git-review --id skill:git-review --tags work`

## lock / fetch / update

- `agentpack lock`: generate/update `agentpack.lock.json`
- `agentpack fetch`: fetch external sources into cache/store per lockfile
- `agentpack update`: composite command
  - Default: run lock+fetch when lockfile is missing; otherwise fetch-only
  - Flags: `--lock`/`--fetch`/`--no-lock`/`--no-fetch`

## preview / plan / diff

- `agentpack plan`: show create/update/delete without applying
- `agentpack diff`: show diffs for the current plan
- `agentpack preview [--diff]`: composite command (always runs plan; also runs diff when `--diff` is set)

Notes:
- Updates in a plan can be one of:
  - `managed_update`: updating a managed file
  - `adopt_update`: overwriting an existing unmanaged file (refused by default; see `deploy --adopt`)

## deploy

`agentpack deploy [--apply] [--adopt]`

- Without `--apply`: show plan + diff only
- With `--apply`: write to target roots, create a snapshot, and update per-root `.agentpack.manifest.json`
- If the plan contains `adopt_update`: you must pass `--adopt` or the command fails with `E_ADOPT_CONFIRM_REQUIRED`

Common:
- `agentpack deploy --apply`
- `agentpack --json deploy --apply --yes`
- `agentpack deploy --apply --adopt`

## status

`agentpack status`
- Detects drift (missing/modified/extra) using `.agentpack.manifest.json`
- If no manifests exist (first run or migration), it falls back to “desired vs FS” and emits a warning

## rollback

`agentpack rollback --to <snapshot_id>`
- Roll back to a deployment/bootstrap snapshot

## doctor

`agentpack doctor [--fix]`
- Checks machine id, target path writability, and common config issues
- `--fix`: idempotently appends `.agentpack.manifest.json` to `.gitignore` for detected git repos (avoid accidental commits)

## remote / sync

- `agentpack remote set <url> [--name origin]`: configure a git remote for the config repo
- `agentpack sync [--rebase] [--remote origin]`: recommended pull/rebase + push sync flow

## bootstrap

`agentpack bootstrap [--scope user|project|both]`
- Installs operator assets:
  - Codex: operator skill
  - Claude Code: `/ap-*` commands

Tip: choose targets via global `--target`:
- `agentpack --target codex bootstrap --scope both`

## overlay

- `agentpack overlay edit <module_id> [--scope global|machine|project] [--sparse|--materialize]`
- `agentpack overlay rebase <module_id> [--scope ...] [--sparsify]` (3-way merge; supports `--dry-run`)
- `agentpack overlay path <module_id> [--scope ...]`

## explain

`agentpack explain plan|diff|status`
- Explains which module and which overlay layer (upstream/global/machine/project) produced a change/drift item.

## record / score

- `agentpack record`: read JSON from stdin and append to `state/logs/events.jsonl`
- `agentpack score`: aggregate events into success/failure stats (skips malformed lines; emits warnings)

## evolve

- `agentpack evolve propose [--module-id <id>] [--scope global|machine|project] [--branch <name>]`
  - Capture drifted deployed content and generate an overlay proposal (creates a branch and writes files)
  - Recommended to start with `--dry-run --json` to inspect candidates
- `agentpack evolve restore [--module-id <id>]`
  - Restore missing desired outputs (create-only; supports `--dry-run`)

## completions

`agentpack completions <shell>`
- Generate shell completion scripts (bash/zsh/fish/powershell, etc.)
