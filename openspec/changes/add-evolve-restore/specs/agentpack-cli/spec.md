# agentpack-cli (delta)

## ADDED Requirements

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
