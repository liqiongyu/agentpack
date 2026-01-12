# agentpack-cli Specification

## Purpose
Define the user-facing CLI contract for `agentpack`: supported commands, stable `--json` envelope behavior, and composite helpers that reduce operational friction. This spec is intentionally API-like so automation (agents/scripts) can depend on consistent command semantics and machine-readable output.
## Requirements
### Requirement: Provide v0.1 command suite
The system SHALL implement the v0.1 CLI commands described in `docs/SPEC.md`, including:
`init`, `add`, `remove`, `lock`, `fetch`, `plan`, `diff`, `deploy`, `status`, `rollback`, `bootstrap`.

#### Scenario: Basic lifecycle in a temp workspace
- **GIVEN** a fresh machine state (empty `AGENTPACK_HOME`)
- **WHEN** the user runs `agentpack init`
- **AND** adds at least one `instructions` and one `command` module
- **AND** runs `agentpack lock` and `agentpack fetch`
- **AND** runs `agentpack deploy --apply`
- **THEN** target outputs are created and discoverable by the configured targets
- **AND** `agentpack status` reports no drift

### Requirement: JSON output contract
When invoked with `--json`, the system SHALL output machine-readable JSON with the stable top-level fields:
`ok`, `command`, `version`, `data`, `warnings`, `errors`.

#### Scenario: plan --json is parseable
- **WHEN** the user runs `agentpack plan --json`
- **THEN** the output is valid JSON
- **AND** includes the required top-level fields

### Requirement: Reproducible lockfile
The system SHALL generate `agentpack.lock.json` with stable ordering and deterministic hashing for resolved module content.

#### Scenario: lockfile is stable across runs
- **GIVEN** the same sources and refs
- **WHEN** the user runs `agentpack lock` twice
- **THEN** the lockfile content is identical

### Requirement: Safe apply and rollback
The system SHALL create deployment snapshots and SHALL be able to rollback to a snapshot id, restoring previous deployed outputs.

#### Scenario: rollback restores previous state
- **GIVEN** two consecutive successful deployments
- **WHEN** the user runs `agentpack rollback --to <first_snapshot>`
- **THEN** deployed outputs match the first snapshot state

### Requirement: Overlay precedence
The system SHALL apply overlays with this priority (low to high):
upstream -> global overlay -> project overlay.

#### Scenario: project overlay overrides global overlay
- **GIVEN** a module file is overridden in both global and project overlays
- **WHEN** the system renders the module for deployment
- **THEN** the deployed output matches the project overlay content

### Requirement: Update composite command
The system SHALL provide `agentpack update` as a composite command that orchestrates `lock` and `fetch` with a sensible default:
- If the lockfile is missing, it runs `lock` then `fetch`.
- If the lockfile exists, it runs `fetch` (unless explicitly overridden).

#### Scenario: update runs lock+fetch when lockfile is missing
- **GIVEN** an `AGENTPACK_HOME` with no `agentpack.lock.json`
- **WHEN** the user runs `agentpack update`
- **THEN** a lockfile is created
- **AND** git sources are fetched/verified in the store

#### Scenario: update --json without --yes is refused
- **GIVEN** a lockfile exists (or `update` would otherwise perform a write step)
- **WHEN** the user runs `agentpack update --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: Preview composite command
The system SHALL provide `agentpack preview` as a read-only composite command that runs `plan` and optionally includes `diff` via `--diff`.

#### Scenario: preview --json includes plan
- **WHEN** the user runs `agentpack preview --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.plan.summary` exists

#### Scenario: preview --diff --json includes plan and diff
- **WHEN** the user runs `agentpack preview --diff --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.plan.summary` exists
- **AND** `data.diff.summary` exists

### Requirement: Overlay path helper
The system SHALL provide `agentpack overlay path` to resolve the overlay directory for a module id and scope without performing writes.

#### Scenario: overlay path --json outputs overlay_dir
- **WHEN** the user runs `agentpack overlay path <module_id> --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.overlay_dir` is present

### Requirement: Bootstrap templates guide the AI-first loop
The system SHALL ship bootstrap operator templates that guide the user through an AI-first loop including `record`, `score`, `explain`, and `evolve propose`.

#### Scenario: bootstrap Codex operator skill references evolve propose
- **WHEN** the user runs `agentpack bootstrap`
- **THEN** the installed Codex operator skill text includes guidance mentioning `agentpack evolve propose`

### Requirement: Mutating command set is centrally maintained
The system SHALL centrally define the set of “mutating” operations/command IDs (writes to disk or git) and use that single source of truth for:
- enforcing `--json` + `--yes` guardrails, and
- self-description output (e.g., `help --json`) when present.

#### Scenario: lock --json without --yes is refused
- **WHEN** the user runs `agentpack lock --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: help --json is self-describing
The system SHALL provide `agentpack help --json` which emits machine-consumable documentation, including:
- a `commands` list describing available commands/subcommands, and
- `mutating_commands` listing mutating command IDs that require `--yes` in `--json` mode.

#### Scenario: help --json returns commands and mutating_commands
- **WHEN** the user runs `agentpack help --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.commands` exists
- **AND** `data.mutating_commands` exists

### Requirement: schema command documents JSON output contract
The system SHALL provide `agentpack schema` which documents:
- the JSON envelope schema, and
- the minimum expected `data` fields for key read commands (at least: `plan`, `diff`, `preview`, `status`).

#### Scenario: schema --json returns envelope schema
- **WHEN** the user runs `agentpack schema --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.envelope` exists

### Requirement: preview --json --diff includes structured per-file diffs
When invoked as `agentpack preview --json --diff`, the system SHALL include a structured diff payload under `data.diff`:
- `summary` (counts)
- `files[]` with, at minimum: `target`, `root`, `path`, `op`, `before_hash`, `after_hash`

`unified` diffs are optional and MAY be omitted; if omitted due to size limits, the system SHOULD add a warning.

#### Scenario: preview --json --diff includes diff.files
- **WHEN** the user runs `agentpack preview --json --diff`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.diff.summary` exists
- **AND** `data.diff.files` exists

### Requirement: Bootstrap operator assets are version-stamped
The system SHALL stamp bootstrap-installed operator assets with an `agentpack_version: x.y.z` marker (frontmatter or comment) matching the running `agentpack` version.

#### Scenario: bootstrap-installed assets include agentpack_version
- **WHEN** the user runs `agentpack bootstrap`
- **THEN** the installed operator assets contain an `agentpack_version` marker

### Requirement: status warns when operator assets are missing or outdated
The system SHALL warn when operator assets for the selected target are missing or have an `agentpack_version` that does not match the running `agentpack` version.

#### Scenario: status warns for missing assets
- **GIVEN** operator assets are not installed for the selected target
- **WHEN** the user runs `agentpack status --json`
- **THEN** `warnings[]` includes a message recommending `agentpack bootstrap`

#### Scenario: status warns for outdated assets
- **GIVEN** operator assets exist but have `agentpack_version` that differs from `agentpack --version`
- **WHEN** the user runs `agentpack status --json`
- **THEN** `warnings[]` includes a message recommending `agentpack bootstrap`

### Requirement: overlay path outputs a filesystem-safe directory
The system SHALL make `agentpack overlay path <module_id>` return an overlay directory that is filesystem-safe on the current platform.

#### Scenario: overlay path is Windows-safe
- **GIVEN** a module id `instructions:base`
- **WHEN** the user runs `agentpack overlay path instructions:base --scope global --json`
- **THEN** `data.overlay_dir` does not contain a path segment with `:` on Windows

### Requirement: CLI implementation is modular
The system SHALL structure the CLI implementation as a set of focused modules (e.g., `src/cli/commands/*`) rather than a single monolithic file, so command behavior and output contracts remain easier to maintain and review.

#### Scenario: Command handlers are localized
- **WHEN** a developer adds or updates a CLI subcommand handler
- **THEN** the change is confined to a dedicated module under `src/cli/commands/` and shared helpers under `src/cli/`
