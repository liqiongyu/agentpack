# agentpack Specification

## Purpose
TBD - created by archiving change update-agentpack-v0-2. Update Purpose after archive.
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
Agentpack MUST provide a `doctor` command that outputs a deterministic `machineId` and validates target paths for existence and writability with actionable guidance.

#### Scenario: Doctor reports path issues
- **WHEN** a configured target directory is missing or not writable
- **THEN** `agentpack doctor` reports the issue and a suggested fix (create directory / adjust permissions / change config)
