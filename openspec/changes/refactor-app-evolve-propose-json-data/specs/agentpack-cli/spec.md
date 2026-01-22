## ADDED Requirements

### Requirement: evolve.propose JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `evolve.propose` JSON `data` payload so that CLI `evolve propose --json` and the MCP `evolve_propose` tool keep equivalent output over time.

#### Scenario: evolve.propose JSON payload remains consistent
- **GIVEN** a repo state that results in `evolve.propose` producing one of: noop / dry-run / created
- **WHEN** the user runs `agentpack evolve propose --json`
- **AND** an MCP client calls the `evolve_propose` tool
- **THEN** both responses include equivalent `data` fields for the corresponding outcome
