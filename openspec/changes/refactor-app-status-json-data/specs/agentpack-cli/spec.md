## ADDED Requirements

### Requirement: status JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `status` JSON `data` payload so that CLI `status --json` and the MCP `status` tool keep equivalent output over time.

#### Scenario: status JSON payload remains consistent
- **GIVEN** a status report that yields drift and one or more suggested `next_actions`
- **WHEN** the user runs `agentpack status --json`
- **AND** an MCP client calls the `status` tool
- **THEN** both responses include equivalent `data` fields (`drift`, `summary`, `summary_by_root`, optional `summary_total`, `next_actions`, `next_actions_detailed`) modulo surface-appropriate flags
