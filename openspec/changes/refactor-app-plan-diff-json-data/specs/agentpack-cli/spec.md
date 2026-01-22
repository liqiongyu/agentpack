## ADDED Requirements

### Requirement: plan/diff JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `plan`/`diff` JSON `data` payload so that CLI `plan --json` / `diff --json` and the MCP `plan` / `diff` tools keep equivalent output over time.

#### Scenario: plan/diff JSON payload remains consistent
- **GIVEN** a repo state that yields a plan with one or more changes
- **WHEN** the user runs `agentpack plan --json` and `agentpack diff --json`
- **AND** an MCP client calls the `plan` and `diff` tools
- **THEN** each corresponding response includes equivalent `data` fields (`profile`, `targets`, `changes`, `summary`) modulo surface-appropriate flags
