# Changelog

All notable changes to this project will be documented in this file.

The format is based on *Keep a Changelog*, and this project adheres to Semantic Versioning.

## [Unreleased]

- TBD

## [0.2.0] - 2026-01-11

- Deploy safety via per-target `.agentpack.manifest.json` (safe deletes + manifest-based drift).
- New commands: `doctor`, `remote set`, `sync`, `record`, `score`, `explain`, `evolve propose`.
- Machine overlays: `overlays/machines/<machineId>/...` + global `--machine`.
- `--json` now includes a `schema_version` field (backwards compatible).
- Default `AGENTPACK_HOME` is now `~/.agentpack` with `cache/` + `state/snapshots/` + `state/logs/`.

## [0.1.0] - 2026-01-10

- Initial end-to-end CLI workflow: `init/add/lock/fetch/plan/diff/deploy/status/rollback/bootstrap`.
- Targets: `codex`, `claude_code`; overlays: global + project; git lockfile + store cache.
