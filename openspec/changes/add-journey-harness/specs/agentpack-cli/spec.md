# agentpack-cli Delta

## ADDED Requirements

### Requirement: Reusable journey test harness
The project SHALL provide a reusable integration test harness under `tests/journeys/common` to support deterministic, offline journey (E2E) tests for the `agentpack` CLI.

#### Scenario: Run commands in an isolated temp environment
- **GIVEN** a journey test constructs a `TestEnv`
- **WHEN** it runs `agentpack init` using the harness command helper
- **THEN** the command runs with temp `HOME` and temp `AGENTPACK_HOME`
- **AND** no host state outside the temp directories is modified
