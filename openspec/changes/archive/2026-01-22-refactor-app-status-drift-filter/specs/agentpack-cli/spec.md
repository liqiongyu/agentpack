## ADDED Requirements

### Requirement: status drift filtering remains consistent across CLI and MCP
The system SHALL centralize the `status --only` drift filtering behavior so that CLI `status --json` and the MCP `status` tool apply equivalent filtering and `summary_total` behavior over time.

#### Scenario: drift filtering remains consistent
- **GIVEN** a drift report containing multiple drift kinds
- **WHEN** the user runs `agentpack status --only modified --json`
- **AND** an MCP client calls the `status` tool with `only: ["modified"]`
- **THEN** both responses include equivalent filtered `data.drift`
- **AND** both include equivalent `data.summary`
- **AND** both include equivalent `data.summary_total`
