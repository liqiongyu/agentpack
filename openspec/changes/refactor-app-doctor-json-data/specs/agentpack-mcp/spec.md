## ADDED Requirements

### Requirement: doctor JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `doctor` JSON `data` payload so that the MCP `doctor` tool stays consistent with CLI `doctor --json` over time.

#### Scenario: doctor JSON payload remains consistent
- **GIVEN** a doctor report that yields one or more suggested `next_actions`
- **WHEN** an MCP client calls the `doctor` tool
- **AND** a user runs `agentpack doctor --json`
- **THEN** both responses include equivalent `data` fields (`machine_id`, `roots`, `gitignore_fixes`, optional `next_actions`) modulo surface-appropriate flags
