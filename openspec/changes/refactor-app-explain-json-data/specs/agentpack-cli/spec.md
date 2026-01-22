## ADDED Requirements

### Requirement: explain JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `explain.plan` and `explain.status` JSON `data` payloads so that CLI `agentpack explain * --json` and the MCP `explain` tool keep equivalent output over time.

#### Scenario: explain.plan JSON payload remains consistent
- **GIVEN** a repo state that produces an `explain.plan` response
- **WHEN** the user runs `agentpack explain plan --json` (or `agentpack explain diff --json`)
- **AND** an MCP client calls the `explain` tool with `kind=plan` (or `kind=diff`)
- **THEN** both responses include equivalent `data` fields (`profile`, `targets`, `changes`)

#### Scenario: explain.status JSON payload remains consistent
- **GIVEN** a repo state that produces an `explain.status` response
- **WHEN** the user runs `agentpack explain status --json`
- **AND** an MCP client calls the `explain` tool with `kind=status`
- **THEN** both responses include equivalent `data` fields (`profile`, `targets`, `drift`)
