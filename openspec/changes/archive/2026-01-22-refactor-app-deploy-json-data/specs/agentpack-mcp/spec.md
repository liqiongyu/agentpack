## ADDED Requirements

### Requirement: deploy JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `deploy` JSON `data` payload so that the MCP `deploy` / `deploy_apply` tools stay consistent with CLI `deploy --json` over time.

#### Scenario: deploy JSON payload remains consistent
- **GIVEN** a deployment plan that yields either no changes or applyable changes
- **WHEN** an MCP client calls `deploy` and `deploy_apply`
- **AND** a user runs `agentpack deploy --json` (optionally with `--apply`)
- **THEN** the responses include equivalent `data` fields (`applied`, optional `reason`, optional `snapshot_id`, `profile`, `targets`, `changes`, `summary`) modulo surface-appropriate flags
