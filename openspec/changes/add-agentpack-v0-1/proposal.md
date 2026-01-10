# Change: Add agentpack v0.1 implementation

## Why
Agentpack needs an end-to-end, reproducible workflow to manage and deploy AI-coding assets (instructions/skills/prompts/commands) across Codex CLI and Claude Code with safety (diff + rollback) and machine-friendly automation (`--json`).

## What Changes
- Implement the `agentpack` CLI per `docs/SPEC.md` (v0.1 command set and `--json` contract).
- Implement core engine components per `docs/ARCHITECTURE.md`:
  - manifest loader, lockfile, store, overlay engine, renderer/compiler
  - deploy pipeline: plan/diff/apply/validate/snapshot/rollback/status
- Implement v0.1 adapters for `codex` and `claude_code`.
- Add tests (unit + golden snapshots) and CI coverage aligned to `docs/BACKLOG.md` (P0).

## Scope
- **In scope**: `codex`, `claude_code` targets; `instructions`, `skill`, `prompt`, `command` module types; `local_path` and `git` sources.
- **Out of scope** (v0.1): plugin output modes, MCP modules, GUI/TUI, cloud sync, automatic 3-way merge overlays, full eval gate.

## Risks & Mitigations
- **Risk**: accidental deletion of user-owned files.
  - **Mitigation**: track managed files in deployment snapshots and only delete files that were previously managed.
- **Risk**: overlay drift when upstream changes.
  - **Mitigation**: record upstream baseline hashes for overlay edits and warn on upstream changes.
- **Risk**: tool discovery rules drift across Codex/Claude versions.
  - **Mitigation**: adapter-level golden tests for `plan` output and filesystem layout.

## Acceptance (high-level)
- All v0.1 commands work in a temp repo and produce stable `--json` output.
- Deploy is idempotent; repeated deploy yields no drift.
- Rollback restores previous deployed outputs.
- CI runs fmt/clippy/test and a security audit.
