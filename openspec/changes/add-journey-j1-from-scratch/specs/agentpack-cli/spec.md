# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J1 is covered by an integration test
The project SHALL include a deterministic, offline integration test for Journey J1 (from-scratch first deploy) that exercises the core CLI lifecycle end-to-end.

#### Scenario: From-scratch deploy flow succeeds end-to-end
- **GIVEN** an empty temp `AGENTPACK_HOME` and temp `HOME`
- **WHEN** the user runs `agentpack init`
- **AND** runs `agentpack update`
- **AND** runs `agentpack preview --diff`
- **AND** runs `agentpack deploy --apply`
- **AND** runs `agentpack status`
- **AND** runs `agentpack rollback --to <snapshot_id>`
- **THEN** each command succeeds
- **AND** `status` reports no drift after deploy
