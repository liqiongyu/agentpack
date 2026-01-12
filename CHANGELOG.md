# Changelog

All notable changes to this project will be documented in this file.

The format is based on *Keep a Changelog*, and this project adheres to Semantic Versioning.

## Versioning policy

This project follows SemVer with the following clarifications (especially for automation users):

- **MAJOR**: Breaking changes (examples)
  - `--json` envelope changes that require bumping `schema_version` (removal/rename/semantic changes of existing fields)
  - Changing the meaning of an existing stable error code
  - Removing/renaming an existing CLI command/flag in a way that breaks automation or documented behavior
- **MINOR**: Backwards-compatible new functionality
  - New commands/flags (additive)
  - New stable error codes (additive)
  - New JSON fields under the same `schema_version` (additive)
- **PATCH**: Bug fixes and improvements
  - Fixes that restore intended behavior
  - Docs and internal refactors that do not change stable contracts

## [Unreleased]

### Added
- Dependency policy checks in CI (`cargo-deny`) and documentation (`docs/SECURITY_CHECKS.md`).
- OpenSSF Scorecard workflow (`.github/workflows/scorecard.yml`).
- Release automation via cargo-dist (`.github/workflows/release.yml`) and docs (`docs/RELEASING.md`).
- `--json` contract docs: `docs/JSON_API.md` and `docs/ERROR_CODES.md`.
- GitHub Issue Forms (`.github/ISSUE_TEMPLATE/*.yml`).

### Changed
- Deploy now requires explicit `--adopt` for `adopt_update` overwrites (stable `E_ADOPT_CONFIRM_REQUIRED`).
- Overlay/store directory naming uses a bounded, filesystem-safe `module_fs_key` with legacy fallbacks.
- Combined Codex instructions output (`AGENTS.md`) uses per-module section markers to enable `evolve propose` mapping for aggregated outputs.

### Fixed
- Unifies atomic writes across all critical file writes.
- Improved human error output for `E_DESIRED_STATE_CONFLICT`.

### Security
- Adds license/source/duplicate dependency checks via `deny.toml` and CI.

## [0.4.0] - 2026-01-11

### Added
- Self-describing automation: `agentpack help --json` and `agentpack schema --json`.
- Better machine diffs: `agentpack preview --json --diff` includes structured `diff.files` with hashes and optional unified diffs.
- Target conformance suite: hermetic smoke tests for `codex` and `claude_code`, plus contributor docs.
- Bootstrap UX: expanded Claude operator commands, version-stamped operator assets, and `status` warnings for missing/outdated installs.

### Changed
- Guardrails hardening: centrally maintained mutating command set; `--json` mutations require `--yes` (including `init`, `bootstrap`, `overlay edit`, `rollback`).
- Target platformization: TargetAdapter registry for rendering and roots.
- Rollback reliability: snapshots store deployed state so rollback restores previous outputs even if drift occurred later.

## [0.3.0] - 2026-01-11

### Added
- New composite commands: `update` (lock + fetch) and `preview` (plan + optional diff).
- Cache miss safety net: auto-fetch missing git checkouts when a lockfile exists.
- Overlays UX: `overlay edit --scope global|machine|project` + helper `overlay path`.
- Git hygiene: `doctor` warns about `.agentpack.manifest.json` in git repos; `doctor --fix` updates `.gitignore`.

### Changed
- Stronger AI-first safety: `--json` write commands require `--yes` (stable `E_CONFIRM_REQUIRED`).

## [0.2.0] - 2026-01-11

### Added
- Deploy safety via per-target `.agentpack.manifest.json` (safe deletes + manifest-based drift).
- New commands: `doctor`, `remote set`, `sync`, `record`, `score`, `explain`, `evolve propose`.
- Machine overlays: `overlays/machines/<machineId>/...` + global `--machine`.
- `--json` now includes a `schema_version` field (backwards compatible).

### Changed
- Default `AGENTPACK_HOME` is now `~/.agentpack` with `cache/` + `state/snapshots/` + `state/logs/`.

## [0.1.0] - 2026-01-10

### Added
- Initial end-to-end CLI workflow: `init/add/lock/fetch/plan/diff/deploy/status/rollback/bootstrap`.
- Targets: `codex`, `claude_code`; overlays: global + project; git lockfile + store cache.
