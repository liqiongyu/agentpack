# Changelog

All notable changes to this project will be documented in this file.

The format is based on *Keep a Changelog*, and this project adheres to Semantic Versioning.

## [Unreleased]

- TBD

## [0.3.0] - 2026-01-11

- New composite commands: `update` (lock + fetch) and `preview` (plan + optional diff).
- Cache miss safety net: auto-fetch missing git checkouts when a lockfile exists.
- Overlays UX: `overlay edit --scope global|machine|project` + helper `overlay path`.
- Stronger AI-first safety: `--json` write commands require `--yes` (stable `E_CONFIRM_REQUIRED`).
- Git hygiene: `doctor` warns about `.agentpack.manifest.json` in git repos; `doctor --fix` updates `.gitignore`.

## [0.2.0] - 2026-01-11

- Deploy safety via per-target `.agentpack.manifest.json` (safe deletes + manifest-based drift).
- New commands: `doctor`, `remote set`, `sync`, `record`, `score`, `explain`, `evolve propose`.
- Machine overlays: `overlays/machines/<machineId>/...` + global `--machine`.
- `--json` now includes a `schema_version` field (backwards compatible).
- Default `AGENTPACK_HOME` is now `~/.agentpack` with `cache/` + `state/snapshots/` + `state/logs/`.

## [0.1.0] - 2026-01-10

- Initial end-to-end CLI workflow: `init/add/lock/fetch/plan/diff/deploy/status/rollback/bootstrap`.
- Targets: `codex`, `claude_code`; overlays: global + project; git lockfile + store cache.
