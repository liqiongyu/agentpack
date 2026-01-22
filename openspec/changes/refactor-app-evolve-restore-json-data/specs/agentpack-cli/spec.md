## ADDED Requirements

### Requirement: evolve.restore JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `evolve.restore` JSON `data` payload so that CLI `evolve restore --json` and the MCP `evolve_restore` tool keep equivalent output over time.

#### Scenario: evolve.restore JSON payload remains consistent
- **GIVEN** a repo state with missing desired outputs (or none)
- **WHEN** the user runs `agentpack evolve restore --json`
- **AND** an MCP client calls the `evolve_restore` tool
- **THEN** both responses include equivalent `data` fields (`restored`, `summary`, `reason`) modulo surface-appropriate flags
