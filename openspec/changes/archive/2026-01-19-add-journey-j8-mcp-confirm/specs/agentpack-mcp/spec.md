# agentpack-mcp Delta

## ADDED Requirements

### Requirement: Journey J8 is covered by an integration test

The project SHALL include a deterministic, offline integration test for Journey J8 (MCP confirm_token) that validates the two-stage deploy/apply flow and rollback using MCP tools.

#### Scenario: deploy returns confirm_token and apply+rollback require explicit approval
- **GIVEN** an Agentpack MCP server is running over stdio
- **WHEN** a client calls `deploy` and receives `data.confirm_token`
- **AND** calls `deploy_apply` without a token / with a mismatched token
- **THEN** the server refuses with stable error codes without writing
- **WHEN** a client calls `deploy_apply` with `yes=true` and a matching `confirm_token`
- **THEN** the deployment applies successfully
- **AND** `rollback` with `yes=true` restores the prior snapshot
