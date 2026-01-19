# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J7 is covered by an integration test

The project SHALL include a deterministic, offline integration test for Journey J7 (cross-target consistency) that validates deploying a single module to multiple targets and rolling back restores all target outputs.

#### Scenario: deploy to multiple targets and rollback restores both
- **GIVEN** a config repo with a single skill module targeting both `codex` and `claude_code`
- **WHEN** the user runs `agentpack --target all deploy --apply --json --yes` twice with different content
- **AND** runs `agentpack --target all rollback --to <snapshot_id> --json --yes`
- **THEN** the deployed outputs for both targets match the first snapshot state
