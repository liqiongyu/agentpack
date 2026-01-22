## ADDED Requirements

### Requirement: evolve.propose JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `evolve.propose` JSON `data` payload so that the MCP `evolve_propose` tool stays consistent with CLI `evolve propose --json` over time.

#### Scenario: evolve.propose JSON payload remains consistent
- **GIVEN** a repo state that results in `evolve.propose` producing one of: noop / dry-run / created
- **WHEN** an MCP client calls the `evolve_propose` tool
- **AND** a user runs `agentpack evolve propose --json`
- **THEN** both responses include equivalent `data` fields for the corresponding outcome
