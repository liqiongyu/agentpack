# CLI reference

> Language: English | [Chinese (Simplified)](../zh-CN/reference/cli.md)

This document is for quickly looking up how a command works. For workflow-oriented guidance, see `../howto/workflows.md`.

## Global flags (supported by all commands)

- `--dry-run`: Force dry-run behavior (do not apply even if --apply is set)
- `--json`: Machine-readable JSON output
- `--machine <machine>`: Machine id for machine overlays (default: auto-detect)
- `--profile <profile>`: Profile name (default: "default")
- `--repo <repo>`: Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)
- `--target <target>`: Target name: codex|claude_code|cursor|vscode|jetbrains|zed|all (default: "all")
- `--yes`: Skip confirmations (dangerous with --apply)

Tips:
- `agentpack help --json` returns a structured command list and the mutating command set.
- `agentpack schema --json` describes the JSON envelope and common `data` payload shapes.

## Commands

All commands below also accept the global flags listed above.

### add

Add a module to agentpack.yaml

Usage: `agentpack add <instructions|skill|prompt|command> <source> [OPTIONS]`

Positional arguments:
- `<instructions|skill|prompt|command>`
- `<source>`: Source spec: local:... or git:...

Options:
- `--id <id>`: Explicit module id (default: derived from type + source)
- `--tags <tags>`: Comma-separated tags (for profiles)
- `--targets <targets>`: Comma-separated target names (codex, claude_code, cursor, vscode, jetbrains, zed). Empty = all

### bootstrap

Install operator assets for AI self-serve

Usage: `agentpack bootstrap [OPTIONS]`

Options:
- `--scope <user|project|both>`: Where to install operator assets (default: both)

### completions

Generate shell completion scripts

Usage: `agentpack completions <bash|elvish|fish|powershell|zsh> [OPTIONS]`

Positional arguments:
- `<bash|elvish|fish|powershell|zsh>`

### deploy

Plan+diff, and optionally apply with --apply

Usage: `agentpack deploy [OPTIONS]`

Options:
- `--adopt`: Allow overwriting existing unmanaged files (adopt updates)
- `--apply`: Apply changes (writes to targets)

### diff

Show diffs for planned changes

Usage: `agentpack diff [OPTIONS]`

### doctor

Check local environment and target paths

Usage: `agentpack doctor [OPTIONS]`

Options:
- `--fix`: Idempotently add `.agentpack.manifest*.json` to `.gitignore` for detected repos

### evolve propose

Propose overlay updates by capturing drifted deployed files into overlays (creates a local git branch)

Usage: `agentpack evolve propose [OPTIONS]`

Options:
- `--branch <branch>`: Branch name to create (default: evolve/propose-<scope>-<module>-<timestamp>)
- `--module-id <module_id>`: Only propose changes for a single module id
- `--scope <global|machine|project>`: Overlay scope to write into (default: global)

### evolve restore

Restore missing desired outputs on disk (create-only; no updates/deletes)

Usage: `agentpack evolve restore [OPTIONS]`

Options:
- `--module-id <module_id>`: Only restore missing outputs attributable to a module id

### explain diff

Explain the current diff (same as plan, with diffs in `agentpack diff`)

Usage: `agentpack explain diff [OPTIONS]`

### explain plan

Explain the current plan (module provenance and overlay layers)

Usage: `agentpack explain plan [OPTIONS]`

### explain status

Explain current drift/status (module provenance and overlay layers)

Usage: `agentpack explain status [OPTIONS]`

### fetch

Fetch sources into store (per lockfile)

Usage: `agentpack fetch [OPTIONS]`

### help

Self-describing CLI help (supports --json)

Usage: `agentpack help [OPTIONS]`

Options:
- `--markdown`: Output a deterministic Markdown reference (for `docs/reference/cli.md`)

### import

Import existing assets into the config repo

Usage: `agentpack import [OPTIONS]`

Options:
- `--home-root <home_root>`: Override the home root used for scanning (default: resolved home dir)
- `--apply`: Apply changes (writes modules + updates manifest)

### init

Initialize the agentpack config repo

Usage: `agentpack init [OPTIONS]`

Options:
- `--bootstrap`: Also install operator assets after init (equivalent to `agentpack bootstrap`)
- `--git`: Also initialize the repo as a git repository (idempotent)
- `--guided`: Interactive wizard (TTY only) to generate a minimal agentpack.yaml

### lock

Generate/update agentpack.lock.json

Usage: `agentpack lock [OPTIONS]`

### mcp serve

Run the Agentpack MCP server over stdio

Usage: `agentpack mcp serve [OPTIONS]`

### overlay edit

Create an overlay skeleton and open an editor

Usage: `agentpack overlay edit <module_id> [OPTIONS]`

Positional arguments:
- `<module_id>`

Options:
- `--kind <dir|patch>`: Overlay kind to create/edit (default: dir)
- `--scope <global|machine|project>`: Overlay scope to write into (default: global)
- `--materialize`: Populate upstream files into the overlay without overwriting existing edits
- `--project`: Use project overlay (DEPRECATED: use --scope project)
- `--sparse`: Create a sparse overlay (do not copy upstream files)

### overlay path

Print the resolved overlay directory for a module and scope

Usage: `agentpack overlay path <module_id> [OPTIONS]`

Positional arguments:
- `<module_id>`

Options:
- `--scope <global|machine|project>`: Overlay scope to resolve (default: global)

### overlay rebase

Rebase an overlay against the current upstream (3-way merge)

Usage: `agentpack overlay rebase <module_id> [OPTIONS]`

Positional arguments:
- `<module_id>`

Options:
- `--scope <global|machine|project>`: Overlay scope to rebase (default: global)
- `--sparsify`: Remove overlay files that end up identical to upstream after rebasing

### plan

Show planned changes without applying

Usage: `agentpack plan [OPTIONS]`

### policy audit

Generate a supply-chain audit report from lockfiles (read-only)

Usage: `agentpack policy audit [OPTIONS]`

### policy lint

Lint operator assets and policy constraints (read-only)

Usage: `agentpack policy lint [OPTIONS]`

### policy lock

Resolve and pin the configured policy pack (writes agentpack.org.lock.json)

Usage: `agentpack policy lock [OPTIONS]`

### preview

Composite command: plan + (optional) diff

Usage: `agentpack preview [OPTIONS]`

Options:
- `--diff`: Include diffs (human: unified diff; json: diff summary)

### record

Record an execution event (reads JSON from stdin and appends to local logs)

Usage: `agentpack record [OPTIONS]`

### remote set

Set a git remote URL for the config repo (creates the remote if missing)

Usage: `agentpack remote set <url> [OPTIONS]`

Positional arguments:
- `<url>`

Options:
- `--name <name>`: Remote name (default: origin)

### remove

Remove a module from agentpack.yaml

Usage: `agentpack remove <module_id> [OPTIONS]`

Positional arguments:
- `<module_id>`

### rollback

Rollback to a deployment snapshot

Usage: `agentpack rollback [OPTIONS]`

Options:
- `--to <to>`: Snapshot id to rollback to

### schema

Describe the JSON output contract (supports --json)

Usage: `agentpack schema [OPTIONS]`

### score

Score modules based on recorded events

Usage: `agentpack score [OPTIONS]`

### status

Check drift between expected and deployed outputs

Usage: `agentpack status [OPTIONS]`

Options:
- `--only <missing|modified|extra>`: Filter drift items by kind (repeatable or comma-separated)

### sync

Sync the agentpack config repo (pull/rebase + push)

Usage: `agentpack sync [OPTIONS]`

Options:
- `--remote <remote>`: Remote name (default: origin)
- `--rebase`: Pull with rebase (recommended)

### update

Composite command: lock and/or fetch (default: fetch; runs lock+fetch if lockfile is missing)

Usage: `agentpack update [OPTIONS]`

Options:
- `--fetch`: Force running fetch
- `--lock`: Force re-generating the lockfile
- `--no-fetch`: Skip fetch
- `--no-lock`: Skip lockfile generation

## Optional commands

### tui

The `tui` command is feature-gated. Build with `--features tui` to enable it. It does not support `--json`.
