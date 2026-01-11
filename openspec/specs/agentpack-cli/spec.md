# agentpack-cli Specification

## Purpose
TBD - created by archiving change add-agentpack-v0-1. Update Purpose after archive.
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
