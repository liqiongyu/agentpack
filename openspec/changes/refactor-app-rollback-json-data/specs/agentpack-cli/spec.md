## ADDED Requirements

### Requirement: rollback JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `rollback` JSON `data` payload so that CLI `rollback --json` and the MCP `rollback` tool keep equivalent output over time.

#### Scenario: rollback JSON payload remains consistent
- **GIVEN** an existing snapshot id that can be rolled back to
- **WHEN** the user runs `agentpack rollback --to <snapshot_id> --json`
- **AND** an MCP client calls the `rollback` tool
- **THEN** both responses include equivalent `data` fields (`rolled_back_to`, `event_snapshot_id`) modulo surface-appropriate flags
