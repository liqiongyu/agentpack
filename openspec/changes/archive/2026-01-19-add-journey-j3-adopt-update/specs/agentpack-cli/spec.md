# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J3 is covered by an integration test
The project SHALL include a deterministic, offline integration test for Journey J3 (adopt-update flow) that verifies refusal without `--adopt` and success after adoption.

#### Scenario: deploy refuses adopt_update unless --adopt, then allows managed updates
- **GIVEN** an existing unmanaged file at a desired output path
- **WHEN** the user runs `agentpack deploy --apply --json --yes` without `--adopt`
- **THEN** the command fails with `errors[0].code = E_ADOPT_CONFIRM_REQUIRED`
- **WHEN** the user re-runs with `--adopt`
- **THEN** the command applies successfully
- **AND** a subsequent update to the same path is treated as `managed_update` (no `--adopt` required)
