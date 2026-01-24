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
- Expanded additive refusal guidance fields (`errors[0].details.reason_code` and `errors[0].details.next_actions`) across more stable error codes (including IO, config/lockfile validation, target selection, policy, overlays, and deploy refusals).

### Changed
- CI now uses `actions/setup-node@v6`.
- MCP dependency `rmcp` updated to v0.14.0.

### Fixed

### Security

## [0.9.0] - 2026-01-23

### Added
- Stable error code `E_GIT_NOT_FOUND` when `git` is missing.
- Additive `command_id` and `command_path` in `--json` envelopes and MCP tool envelopes.
- Additive `errors[0].details.reason_code` and `errors[0].details.next_actions` on confirmation-related refusals.

## [0.8.0] - 2026-01-20

### Added
- `agentpack help --markdown` for generating the CLI reference.

### Changed
- MCP tools now run in-process (no subprocess spawning) for improved reliability and performance.
- Governance policy tooling enforces `supply_chain_policy.allowed_git_remotes` for `policy_pack` sources.

### Fixed
- Ignore `.agentpack/.git` directories during copy-tree filtering.

### Security
- Restrict policy packs to allowlisted git remotes to harden the trust chain.

## [0.7.0] - 2026-01-18

### Added
- New built-in targets: `jetbrains` and `zed`.

### Changed
- Target manifests are now written as `.agentpack.manifest.<target>.json` to avoid collisions when multiple targets manage the same root.

## [0.6.0] - 2026-01-15

### Added
- Governance policy tooling: `agentpack policy lint` (CI-friendly), plus `agentpack policy lock` to pin policy packs into `repo/agentpack.org.lock.json`.
- Org distribution policy (opt-in): `policy lint` can enforce required targets/modules when `repo/agentpack.org.yaml` configures `distribution_policy`.
- MCP server integration: `agentpack mcp serve` exposes structured read-only tools (`plan`, `diff`, `status`, `doctor`) plus approval-gated mutations (`deploy_apply`, `rollback`).
- Optional lightweight TUI: `agentpack tui` (feature-gated) for browsing `plan/diff/status` and triggering safe apply with explicit confirmation.

### Changed
- Target adapters are now feature-gated at compile time; `agentpack help --json` reports the compiled target set.

## [0.5.0] - 2026-01-12

### Added
- Dependency policy checks in CI (`cargo-deny`) and documentation (`docs/SECURITY_CHECKS.md`).
- OpenSSF Scorecard workflow (`.github/workflows/scorecard.yml`).
- Release automation via cargo-dist (`.github/workflows/release.yml`) and docs (`docs/RELEASING.md`).
- `--json` contract docs: `docs/JSON_API.md` and `docs/ERROR_CODES.md`.
- GitHub Issue Forms (`.github/ISSUE_TEMPLATE/*.yml`).
- `agentpack evolve restore` to restore missing desired outputs (create-only).
- `agentpack overlay edit --sparse` to create overlays without copying full upstream trees, plus `--materialize` as an opt-in escape hatch.
- `agentpack overlay rebase` to 3-way merge overlays against upstream updates (optional `--sparsify`).

### Changed
- Deploy now requires explicit `--adopt` for `adopt_update` overwrites (stable `E_ADOPT_CONFIRM_REQUIRED`).
- Overlay/store directory naming uses a bounded, filesystem-safe `module_fs_key` with legacy fallbacks.
- Combined Codex instructions output (`AGENTS.md`) uses per-module section markers to enable `evolve propose` mapping for aggregated outputs.

### Fixed
- Unifies atomic writes across all critical file writes.
- Improved human error output for `E_DESIRED_STATE_CONFLICT`.
- Fixes Scorecard workflow action version pin.

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
