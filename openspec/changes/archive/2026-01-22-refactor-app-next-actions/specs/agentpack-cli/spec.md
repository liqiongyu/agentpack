## ADDED Requirements

### Requirement: status next_actions ordering is consistent across CLI and MCP
The system SHALL centralize the `next_actions` ordering and `next_actions_detailed.action` mapping logic so that CLI `status --json` and the MCP `status` tool remain consistent over time.

#### Scenario: next_actions and next_actions_detailed remain consistent
- **GIVEN** the same repo/profile/target inputs
- **WHEN** the user runs `agentpack status --json`
- **AND** an MCP client calls the `status` tool
- **THEN** both responses include equivalent ordering for `data.next_actions`
- **AND** both use the same `next_actions_detailed.action` codes for equivalent commands
