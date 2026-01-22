## ADDED Requirements

### Requirement: status next_actions_detailed remains consistent across CLI and MCP
The system SHALL centralize the construction of `next_actions` and `next_actions_detailed` so that the MCP `status` tool stays consistent with CLI `status --json` over time.

#### Scenario: status next_actions_detailed remains consistent
- **GIVEN** a status report that yields one or more suggested `next_actions`
- **WHEN** an MCP client calls the `status` tool
- **AND** a user runs `agentpack status --json`
- **THEN** both responses include equivalent `next_actions` ordering and `next_actions_detailed.action` codes (modulo surface-appropriate flags)
