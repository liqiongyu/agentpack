## ADDED Requirements

### Requirement: evolve.restore JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `evolve.restore` JSON `data` payload so that the MCP `evolve_restore` tool stays consistent with CLI `evolve restore --json` over time.

#### Scenario: evolve.restore JSON payload remains consistent
- **GIVEN** a repo state with missing desired outputs (or none)
- **WHEN** an MCP client calls the `evolve_restore` tool
- **AND** a user runs `agentpack evolve restore --json`
- **THEN** both responses include equivalent `data` fields (`restored`, `summary`, `reason`) modulo surface-appropriate flags
