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

### Requirement: init can optionally initialize a git repo
The system SHALL support `agentpack init --git` to initialize the created repo directory as a git repository and to write/update a minimal `.gitignore` file.

#### Scenario: init --git creates a git-backed repo skeleton
- **GIVEN** a fresh machine state (empty `AGENTPACK_HOME`)
- **WHEN** the user runs `agentpack init --git`
- **THEN** the repo directory contains a `.git/` directory
- **AND** the repo directory contains `.gitignore` that ignores `.agentpack.manifest.json`

### Requirement: init can optionally bootstrap operator assets
The system SHALL support `agentpack init --bootstrap` to install operator assets into the config repo immediately after initializing the repo (equivalent to running `agentpack bootstrap --scope project`).

#### Scenario: init --bootstrap installs operator assets
- **GIVEN** a fresh machine state (empty `AGENTPACK_HOME`)
- **WHEN** the user runs `agentpack init --bootstrap`
- **THEN** operator assets are installed into configured target locations

### Requirement: JSON output contract
When invoked with `--json`, the system SHALL output machine-readable JSON with the stable top-level fields:
`schema_version`, `ok`, `command`, `version`, `data`, `warnings`, `errors`.

#### Scenario: plan --json is parseable
- **WHEN** the user runs `agentpack plan --json`
- **THEN** the output is valid JSON
- **AND** includes the required top-level fields (including `schema_version`)

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
upstream -> global overlay -> machine overlay -> project overlay.

If `--machine <machine_id>` is not provided, the machine overlay layer SHALL be skipped.

#### Scenario: machine overlay overrides global overlay
- **GIVEN** a module file is overridden in both global and machine overlays
- **WHEN** the user runs `agentpack plan --machine <machine_id>`
- **THEN** the planned content matches the machine overlay content

#### Scenario: project overlay overrides machine overlay
- **GIVEN** a module file is overridden in both machine and project overlays
- **WHEN** the user runs `agentpack plan --machine <machine_id>`
- **THEN** the planned content matches the project overlay content

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

Each item in `data.commands[]` SHALL include:
- `id` (stable command id),
- `path[]` (command path segments),
- `mutating` (whether the base invocation mutates), and
- `supports_json` (whether the command supports `--json` output).

`data.commands[]` SHOULD include `args[]` describing command-specific arguments (excluding global args).

The output SHOULD also include `global_args[]` describing global flags/options.

#### Scenario: help --json returns command metadata
- **WHEN** the user runs `agentpack help --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.commands` exists
- **AND** `data.commands[0].supports_json` exists
- **AND** `data.global_args` exists
- **AND** `data.mutating_commands` exists

### Requirement: schema command documents JSON output contract
The `agentpack schema --json` payload SHALL document `next_actions` as an additive `status` data field.

#### Scenario: schema lists status next_actions field
- **WHEN** the user runs `agentpack schema --json`
- **THEN** the schema output documents `status` data fields including `next_actions`

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

### Requirement: JSON outputs include POSIX-style path fields
When a `--json` payload includes a filesystem path, the system SHALL also emit a POSIX-style companion field (using `/` separators) so cross-platform automation can parse paths consistently.

This change MUST be additive: existing fields remain unchanged in `schema_version=1`.

#### Scenario: plan --json includes POSIX path fields
- **WHEN** the user runs `agentpack plan --json`
- **THEN** each change item includes a POSIX-style path field (e.g. `path_posix`)

### Requirement: evolve restore repairs missing desired outputs
The system SHALL provide `agentpack evolve restore` to repair `missing` drift by restoring the desired outputs on disk.

The command SHALL only create missing files:
- It MUST NOT modify existing files (no updates).
- It MUST NOT delete files.

#### Scenario: evolve restore recreates a missing file
- **GIVEN** a desired output path is missing on disk
- **WHEN** the user runs `agentpack evolve restore --yes`
- **THEN** the missing file is created with the desired content

#### Scenario: evolve restore --json requires --yes
- **WHEN** the user runs `agentpack evolve restore --json` without `--yes`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: overlay edit supports sparse overlays
The system SHALL support creating a sparse overlay via `agentpack overlay edit --sparse`, which creates the overlay directory and required metadata without copying the entire upstream tree.

#### Scenario: overlay edit --sparse creates metadata only
- **GIVEN** a module `<moduleId>` exists and resolves to an upstream root
- **WHEN** the user runs `agentpack overlay edit <moduleId> --scope global --sparse`
- **THEN** the overlay directory exists
- **AND** it contains `.agentpack/baseline.json`
- **AND** it contains `.agentpack/module_id`
- **AND** it does not contain copied upstream files by default

### Requirement: overlay edit supports materializing upstream files
The system SHALL support materializing upstream files into an overlay directory via `agentpack overlay edit --materialize` without overwriting existing overlay edits.

#### Scenario: overlay edit --materialize does not overwrite edits
- **GIVEN** an overlay directory exists with an edited file
- **WHEN** the user runs `agentpack overlay edit <moduleId> --materialize`
- **THEN** upstream files missing from the overlay are copied in
- **AND** existing overlay files are not overwritten

### Requirement: overlay rebase updates overlays against upstream changes
The system SHALL provide `agentpack overlay rebase <module_id>` to update an existing overlay against the current upstream module content.

The command SHALL:
- use `<overlay_dir>/.agentpack/baseline.json` as the merge base,
- merge upstream changes into overlay edits using a 3-way merge, and
- update overlay baseline metadata after a successful rebase.

#### Scenario: rebase updates an unmodified file copy
- **GIVEN** an overlay contains an unmodified copy of an upstream file (identical to the baseline)
- **AND** the upstream file changes
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --scope global`
- **THEN** the overlay file is updated so it no longer pins the old upstream content

#### Scenario: rebase conflict yields stable error code
- **GIVEN** an overlay edits a file that is also changed upstream since the baseline
- **AND** the changes overlap and cannot be merged cleanly
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --json --yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_OVERLAY_REBASE_CONFLICT`

### Requirement: status emits actionable next_actions (additive)
When invoked as `agentpack status --json`, the system SHALL include an additive `data.next_actions` field that suggests follow-up commands.

`data.next_actions` SHALL be a list of command strings (`string[]`), each describing a safe follow-up command the user/agent can run.

This change MUST be additive for `schema_version=1` (no rename/remove of existing fields).

#### Scenario: status suggests bootstrap when operator assets are missing
- **GIVEN** operator assets are missing for the selected target/scope
- **WHEN** the user runs `agentpack status --json`
- **THEN** `data.next_actions[]` includes an action that runs `agentpack bootstrap`

#### Scenario: status suggests deploy --apply when desired-state drift exists
- **GIVEN** `status` detects `modified` or `missing` drift
- **WHEN** the user runs `agentpack status --json`
- **THEN** `data.next_actions[]` includes an action that runs `agentpack deploy --apply`

### Requirement: status supports --only drift filters
The system SHALL support `agentpack status --only <missing|modified|extra>` to filter drift items by kind. The `--only` option SHALL accept repeated values and comma-separated lists.

When invoked with `--json` and `--only` is set, the system SHALL include an additive `data.summary_total` capturing the unfiltered totals.

#### Scenario: status --only missing filters drift and includes totals
- **GIVEN** drift includes at least one `missing` item and at least one non-`missing` item
- **WHEN** the user runs `agentpack status --only missing --json`
- **THEN** every `data.drift[]` item has `kind = "missing"`
- **AND** `data.summary.missing > 0`
- **AND** `data.summary.modified = 0`
- **AND** `data.summary.extra = 0`
- **AND** `data.summary_total.missing > 0`
- **AND** `data.summary_total.modified > 0` OR `data.summary_total.extra > 0`

### Requirement: skill modules require valid SKILL.md frontmatter
The system SHALL validate `skill` modules’ `SKILL.md` during module materialization. `SKILL.md` SHALL start with YAML frontmatter (`--- ... ---`) and the frontmatter SHALL include:
- `name` as a non-empty string
- `description` as a non-empty string

When invoked with `--json`, invalid skill frontmatter SHALL fail with stable code `E_CONFIG_INVALID` and include details identifying the module and field(s) to fix.

#### Scenario: plan --json fails on invalid SKILL.md frontmatter
- **GIVEN** a `skill` module whose `SKILL.md` is missing YAML frontmatter
- **WHEN** the user runs `agentpack plan --target codex --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIG_INVALID`
