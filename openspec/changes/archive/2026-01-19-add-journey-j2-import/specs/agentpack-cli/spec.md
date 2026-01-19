# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J2 is covered by an integration test
The project SHALL include a deterministic, offline integration test for Journey J2 (import existing assets) that verifies dry-run safety and successful apply behavior.

#### Scenario: import dry-run does not write and apply imports modules
- **GIVEN** existing user/project assets on disk
- **WHEN** the user runs `agentpack import` (dry-run)
- **THEN** the command reports planned creates but does not write to the config repo
- **WHEN** the user runs `agentpack import --apply --yes --json`
- **THEN** the command writes imported modules into the config repo and updates the manifest
- **AND** preview/deploy succeeds for the imported assets (including project profile when needed)
