## ADDED Requirements

### Requirement: status drift summaries are consistent across CLI and MCP
The system SHALL centralize status drift summary computations (`summary`, `summary_by_root`) so that CLI `status --json` and the MCP `status` tool remain consistent over time.

#### Scenario: drift summaries remain consistent
- **GIVEN** the same repo/profile/target inputs and drift state
- **WHEN** the user runs `agentpack status --json`
- **AND** an MCP client calls the `status` tool
- **THEN** both responses include equivalent `data.summary`
- **AND** both responses include equivalent `data.summary_by_root` ordering and counts
