# Change: add `agentpack import` command

## Why
Agentpack v0.6 already provides a safe, deterministic engine for deploying assets from a config repo, but first-time adoption is still high-friction: most users already have existing assets living outside the config repo (e.g. `AGENTS.md`, `~/.codex/prompts`, `~/.codex/skills`, `.claude/commands`). A first-class `import` command makes adoption a one-session workflow: scan -> plan -> write modules -> deploy.

## What Changes
- Add a new CLI command `agentpack import` (default: dry-run) that scans existing assets and produces an import plan.
- `agentpack import --apply` writes imported assets into the config repo as `local_path` modules and updates `agentpack.yaml` (no target writes).
- Support a `--home-root <path>` override so tests/CI can avoid scanning the real `~`.
- Define a stable `--json` payload for `command="import"` (additive within `schema_version=1`).

## Impact
- Affected specs:
  - `agentpack-cli` (new command + JSON payload)
- Affected code (planned):
  - `src/cli/args.rs` (CLI wiring)
  - `src/cli/commands/import.rs` (handler)
  - helpers for scanning + deterministic plan output
- Backwards compatibility:
  - additive-only (new command, no changes to existing command semantics)
