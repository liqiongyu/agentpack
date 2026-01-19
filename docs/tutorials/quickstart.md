# Quickstart

> Language: English | [Chinese (Simplified)](../zh-CN/tutorials/quickstart.md)

Goal: get from 0 → your first successful deploy, and understand the key safety guardrails (no accidental deletes/overwrites).

## TL;DR: the daily loop

The shortest path you’ll repeat day-to-day:

1. `agentpack init`
2. `agentpack add ...` (repeat as needed)
3. `agentpack update`
4. `agentpack preview --diff`
5. `agentpack deploy --apply`
6. `agentpack status`
7. `agentpack rollback --to <snapshot_id>` (only when you need to undo)

## Typical setups (multi-tool, multi-scope)

Common “heavy user” setups (pick one to start):

1) **Codex (user) + Claude Code (project)**
- Use Codex for global/user-level assets (skills, prompts, global `AGENTS.md`).
- Use Claude Code for repo/project-level assets (slash commands under `.claude/commands`).

2) **Codex (user + project)**
- Use Codex for both user assets and repo-local assets (repo skills, repo `AGENTS.md`).

3) **VS Code (project) + Cursor (project)**
- Use VS Code + Cursor for repo-local instructions/config, shared via git with your project.

## 0) Install

- Rust users:
  - From crates.io:
    - `cargo install agentpack --locked`
  - If you see `could not find \`agentpack\` in registry \`crates-io\``, install from source:
    - Latest tagged release (recommended): `cargo install --git https://github.com/liqiongyu/agentpack --tag v0.7.0 --locked`
    - Bleeding edge (`main`): `cargo install --git https://github.com/liqiongyu/agentpack --branch main --locked`
- Non-Rust users:
  - Download a prebuilt binary from GitHub Releases (see the repository README).

Verify:
- `agentpack help`

## 1) Initialize your config repo

By default, Agentpack creates/uses the config repo at `~/.agentpack/repo` (override via `AGENTPACK_HOME` or `--repo`).

1. Initialize a skeleton:
- `agentpack init` (or `agentpack init --git`)

This creates:
- `agentpack.yaml` (manifest)
- `modules/` (example module dirs: instructions/prompts/claude-commands)

We recommend turning it into a real git repo:
- Easiest: `agentpack init --git`
- Or manually: `cd ~/.agentpack/repo && git init && git add . && git commit -m "init agentpack"`

Optional: set a remote and sync across machines:
- `agentpack remote set <your_git_url>`
- `agentpack sync --rebase`

Optional: install operator assets (Codex operator skill + Claude commands):
- `agentpack init --bootstrap` (installs into the config repo)
- or run later: `agentpack bootstrap`

## 2) Configure targets (Codex / Claude Code)

`agentpack init` writes a usable default `targets:` config. You usually only need to tweak `options`.

Common minimal recommendations:
- Codex: write user skills, repo skills, global/repo `AGENTS.md`, and user prompts (prompts are user-scope only)
- Claude Code: write repo commands + user commands (skills are off by default; enable if needed)
- Cursor / VS Code: typically project-scope (commit the generated files into the repo when appropriate)

Run a self-check:
- `agentpack doctor`

If you see warnings about permissions or missing directories, create the directories or fix paths/options in `agentpack.yaml` as suggested.

## 3) Add modules

Modules live under `agentpack.yaml -> modules:`. You can edit YAML by hand, but using CLI commands is recommended (fewer footguns):

- Add an instructions module (directory module):
  - `agentpack add instructions local:modules/instructions/base --id instructions:base --tags base`

- Add a Codex prompt (single-file module):
  - `agentpack add prompt local:modules/prompts/draftpr.md --id prompt:draftpr --tags base`

- Add a Claude slash command (single-file module):
  - `agentpack add command local:modules/claude-commands/ap-plan.md --id command:ap-plan --tags base --targets claude_code`

You can also add git modules (locked to a commit for reproducibility):
- `agentpack add skill git:https://github.com/your-org/agentpack-modules.git#ref=v1.2.0&subdir=skills/git-review --id skill:git-review --tags work`

## 4) Lock and fetch dependencies (update)

Recommended composite command:
- `agentpack update`

Behavior:
- If `agentpack.lock.json` does not exist: runs `lock` then `fetch`
- If it exists: runs `fetch` only by default

## 5) Preview

First, see what would change:
- `agentpack preview --diff`

Typical output includes:
- Which targets will be written
- Which files would be created/updated/deleted
- Diffs (when `--diff` is set)

Tip: preview only a specific profile or target:
- `agentpack --profile work preview --diff`
- `agentpack --target codex preview --diff`

## 6) Deploy

To actually write files, you must pass `--apply`:
- `agentpack deploy --apply`

Safety model:
- Deletes only remove **managed files** (tracked via `.agentpack.manifest.<target>.json` per target root), never arbitrary user files.
- Overwrite protection: if a destination path exists but is **not managed**, it is classified as `adopt_update` and is refused by default.

If you want to explicitly take over and overwrite those unmanaged existing files:
- `agentpack deploy --apply --adopt`

If you use `--json` in automation:
- All mutating commands require explicit `--yes`
  - `agentpack --json deploy --apply --yes`
  - If there are `adopt_update`s, also add `--adopt`

## 7) Drift (status) and rollback

- Check drift:
  - `agentpack status`

- Roll back to a previous snapshot:
  - `agentpack rollback --to <snapshot_id>`

The snapshot id is printed after a successful `deploy --apply` (in JSON: `data.snapshot_id`).

## 8) Capture local edits with overlays

When you want to customize an upstream module locally (and still be able to merge upstream updates later), use overlays:

- Create/edit an overlay (default: global scope):
  - `agentpack overlay edit <module_id>`

- Multi-machine setups:
  - Use **machine overlays** for per-machine differences (e.g., different install locations, different tool homes):
    - `agentpack overlay edit <module_id> --scope machine`
  - The machine id defaults to auto-detection; override via `--machine <id>` when needed.

- Multi-project setups:
  - Use **project overlays** for per-project tweaks that shouldn’t affect other repos:
    - `agentpack overlay edit <module_id> --scope project`

- Recommended: create a sparse overlay (don’t copy the entire upstream tree; keep only changed files):
  - `agentpack overlay edit <module_id> --sparse`

- When you need to browse upstream files, materialize missing-only (does not overwrite your edits):
  - `agentpack overlay edit <module_id> --materialize`

- After upstream updates, rebase (3-way merge) your overlay onto the new upstream:
  - `agentpack overlay rebase <module_id> --sparsify`

## 9) AI-first bootstrap (operator assets)

Let agents learn how to use agentpack (recommended once):
- `agentpack bootstrap --scope both`

It installs:
- Codex: an operator skill (teaches Codex to call agentpack CLI, preferring `--json`)
- Claude Code: a set of `/ap-*` slash commands (plan/deploy/status/propose, etc.)

See `BOOTSTRAP.md` for details.

## 10) MCP (Codex integration)

If you want Codex to call Agentpack as an MCP tool server (instead of calling the CLI directly), see `MCP.md`.
