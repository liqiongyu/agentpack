# agentpack Specification

## Purpose
Define the core safety and state-management guarantees of Agentpack’s deployment engine. This spec covers properties that protect user-owned files (target manifests + safe deletes), enable recoverability (snapshots/rollback), and support multi-machine and multi-scope customization (sync and overlays). It also defines critical operator guardrails for automation, especially in `--json` mode.
## Requirements
### Requirement: Target Manifest
`agentpack deploy --apply` MUST write a `.agentpack.manifest.json` file into each managed target root directory that records the managed files and their hashes.

#### Scenario: Manifest exists after deploy
- **WHEN** `agentpack deploy --apply` completes successfully
- **THEN** each target root that received files contains `.agentpack.manifest.json`
- **AND** the manifest lists every managed file with a stable relative path and `sha256` hash

### Requirement: Safe Deletes
Agentpack MUST NOT delete files that are not listed in the target manifest.

#### Scenario: Unmanaged files are preserved
- **GIVEN** a target root contains user-created files not present in the manifest
- **WHEN** `agentpack deploy --apply` runs
- **THEN** those unmanaged files remain untouched

### Requirement: Manifest-Based Status
`agentpack status` MUST compute drift using the target manifest and report managed-file drift as `changed` or `missing`, and unmanaged files as `extra` (non-fatal).

#### Scenario: Status detects drift and extras
- **GIVEN** a deployed target root with a valid manifest
- **WHEN** a managed file is modified, and an unmanaged file is added
- **THEN** `agentpack status` reports `changed` for the managed file and `extra` for the unmanaged file

### Requirement: Multi-Machine Sync
Agentpack MUST provide `remote set` and `sync` commands to standardize recommended git workflows for the agentpack config repo.

#### Scenario: Sync wraps pull/rebase/push
- **WHEN** `agentpack sync --rebase` runs in a git-backed config repo with a configured remote
- **THEN** it performs a pull with rebase and then pushes (or clearly reports failures without modifying history)

### Requirement: Machine Overlays
Agentpack MUST support an optional machine overlay layer between global and project overlays, selectable via a global `--machine` flag.

#### Scenario: Machine overlay affects desired state
- **GIVEN** a file is overridden in `overlays/machines/<machineId>/...`
- **WHEN** `agentpack plan --machine <machineId>` runs
- **THEN** the planned content reflects the machine overlay override

### Requirement: Doctor Self-Check
Agentpack MUST provide a `doctor` command that outputs a deterministic `machineId` and validates target paths for existence and writability with actionable guidance. Additionally, when a target root is inside a git repository, `doctor` MUST warn if `.agentpack.manifest.json` is not ignored, and `doctor --fix` MUST be able to idempotently add it to `.gitignore`.

#### Scenario: Doctor warns about committing target manifests
- **GIVEN** a target root inside a git repository
- **AND** `.agentpack.manifest.json` is not ignored
- **WHEN** the user runs `agentpack doctor`
- **THEN** the output includes a warning recommending adding it to `.gitignore`

### Requirement: JSON-mode write confirmation
When invoked with `--json`, any command that performs writes (filesystem or git) MUST require an explicit `--yes` confirmation. If `--yes` is missing, the system MUST return a JSON error with a stable code `E_CONFIRM_REQUIRED` and MUST NOT perform the write.

#### Scenario: add --json without --yes is refused
- **WHEN** the user runs `agentpack add ... --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: Reproducible materialization from lockfile
When a module is resolved from `agentpack.lock.json` and its git checkout directory is missing locally, the system MUST automatically populate the missing checkout (safe network fetch) or fail with an actionable error instructing the user to run `agentpack fetch/update`.

#### Scenario: Missing checkout is auto-fetched
- **GIVEN** `agentpack.lock.json` pins a git module to commit `<commit>`
- **AND** the local checkout directory for `<moduleId, commit>` does not exist
- **WHEN** the system materializes that module as part of `plan/diff/deploy`
- **THEN** the missing checkout is populated automatically

### Requirement: Overlay editing
The system MUST support creating overlay skeletons across overlay scopes:
- `global`: `repo/overlays/<moduleId>/...`
- `machine`: `repo/overlays/machines/<machineId>/<moduleId>/...`
- `project`: `repo/projects/<projectId>/overlays/<moduleId>/...`

#### Scenario: Machine overlay skeleton is created
- **GIVEN** a module `<moduleId>` exists and resolves to an upstream root
- **WHEN** the user runs `agentpack overlay edit <moduleId> --scope machine`
- **THEN** the directory `repo/overlays/machines/<machineId>/<moduleId>/` exists
- **AND** it contains the upstream content (copied)
- **AND** it contains `.agentpack/baseline.json`

### Requirement: JSON-mode write confirmation covers all write-capable commands
When invoked with `--json`, any command that performs writes (filesystem or git) MUST require an explicit `--yes` confirmation. If `--yes` is missing, the system MUST return a JSON error with a stable code `E_CONFIRM_REQUIRED` and MUST NOT perform the write.

This includes (at minimum): `init`, `lock`, `fetch`, `overlay edit`, `remote set`, `sync`, `record`, and `rollback` (in addition to existing covered commands like `add/remove/update/deploy/bootstrap/doctor --fix/evolve propose`).

#### Scenario: init --json without --yes is refused
- **WHEN** the user runs `agentpack init --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

#### Scenario: overlay edit --json without --yes is refused
- **WHEN** the user runs `agentpack overlay edit <moduleId> --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex` and `claude_code` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths
