## ADDED Requirements

### Requirement: status next_actions suggestions remain consistent across CLI and MCP
The system SHALL centralize the status `next_actions` suggestion logic so that the MCP `status` tool stays consistent with CLI `status --json` over time.

#### Scenario: status next_actions suggestions remain consistent
- **GIVEN** a drift summary that indicates `extra` changes but no `modified` or `missing`
- **WHEN** an MCP client calls the `status` tool
- **AND** a user runs `agentpack status --json`
- **THEN** both responses include equivalent suggested `next_actions` (modulo surface-appropriate `--json` / `--yes` flags)
