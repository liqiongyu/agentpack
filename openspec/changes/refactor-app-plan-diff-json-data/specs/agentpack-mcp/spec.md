## ADDED Requirements

### Requirement: plan/diff JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `plan`/`diff` JSON `data` payload so that the MCP `plan` / `diff` tools stay consistent with CLI `plan --json` / `diff --json` over time.

#### Scenario: plan/diff JSON payload remains consistent
- **GIVEN** a repo state that yields a plan with one or more changes
- **WHEN** an MCP client calls the `plan` and `diff` tools
- **AND** a user runs `agentpack plan --json` and `agentpack diff --json`
- **THEN** each corresponding response includes equivalent `data` fields (`profile`, `targets`, `changes`, `summary`) modulo surface-appropriate flags
