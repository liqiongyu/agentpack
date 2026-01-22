## ADDED Requirements

### Requirement: status next_actions_detailed remains consistent across CLI and MCP
The system SHALL centralize the construction of `next_actions` and `next_actions_detailed` so that CLI `status --json` and the MCP `status` tool keep equivalent output over time.

#### Scenario: status next_actions_detailed remains consistent
- **GIVEN** a status report that yields one or more suggested `next_actions`
- **WHEN** the user runs `agentpack status --json`
- **AND** an MCP client calls the `status` tool
- **THEN** both responses include equivalent `next_actions` ordering and `next_actions_detailed.action` codes (modulo surface-appropriate flags)
