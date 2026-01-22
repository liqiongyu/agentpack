## ADDED Requirements

### Requirement: doctor JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `doctor` JSON `data` payload so that CLI `doctor --json` and the MCP `doctor` tool keep equivalent output over time.

#### Scenario: doctor JSON payload remains consistent
- **GIVEN** a doctor report that yields one or more suggested `next_actions`
- **WHEN** the user runs `agentpack doctor --json`
- **AND** an MCP client calls the `doctor` tool
- **THEN** both responses include equivalent `data` fields (`machine_id`, `roots`, `gitignore_fixes`, optional `next_actions`) modulo surface-appropriate flags
