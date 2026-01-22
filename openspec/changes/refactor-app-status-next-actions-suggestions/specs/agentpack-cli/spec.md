## ADDED Requirements

### Requirement: status next_actions suggestions remain consistent across CLI and MCP
The system SHALL centralize the status `next_actions` suggestion logic so that CLI `status --json` and the MCP `status` tool keep equivalent suggestion behavior over time.

#### Scenario: status next_actions suggestions remain consistent
- **GIVEN** a drift summary that indicates `modified` and/or `missing` changes
- **WHEN** the user runs `agentpack status --json`
- **AND** an MCP client calls the `status` tool
- **THEN** both responses include equivalent suggested `next_actions` (modulo surface-appropriate `--json` / `--yes` flags)
