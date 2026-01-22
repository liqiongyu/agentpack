## ADDED Requirements

### Requirement: status JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `status` JSON `data` payload so that the MCP `status` tool stays consistent with CLI `status --json` over time.

#### Scenario: status JSON payload remains consistent
- **GIVEN** a status report that yields drift and one or more suggested `next_actions`
- **WHEN** an MCP client calls the `status` tool
- **AND** a user runs `agentpack status --json`
- **THEN** both responses include equivalent `data` fields (`drift`, `summary`, `summary_by_root`, optional `summary_total`, `next_actions`, `next_actions_detailed`) modulo surface-appropriate flags
