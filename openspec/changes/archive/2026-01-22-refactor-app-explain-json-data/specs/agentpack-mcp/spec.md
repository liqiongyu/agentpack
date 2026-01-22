## ADDED Requirements

### Requirement: explain JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `explain.plan` and `explain.status` JSON `data` payloads so that the MCP `explain` tool stays consistent with CLI `agentpack explain * --json` over time.

#### Scenario: explain.plan JSON payload remains consistent
- **GIVEN** a repo state that produces an `explain.plan` response
- **WHEN** an MCP client calls the `explain` tool with `kind=plan` (or `kind=diff`)
- **AND** a user runs `agentpack explain plan --json` (or `agentpack explain diff --json`)
- **THEN** both responses include equivalent `data` fields (`profile`, `targets`, `changes`)

#### Scenario: explain.status JSON payload remains consistent
- **GIVEN** a repo state that produces an `explain.status` response
- **WHEN** an MCP client calls the `explain` tool with `kind=status`
- **AND** a user runs `agentpack explain status --json`
- **THEN** both responses include equivalent `data` fields (`profile`, `targets`, `drift`)
