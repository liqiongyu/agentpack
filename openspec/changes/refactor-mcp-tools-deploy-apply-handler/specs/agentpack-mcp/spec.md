## ADDED Requirements

### Requirement: MCP deploy_apply applies in-process
The system SHALL compute and apply MCP tool `deploy_apply` in-process (without spawning an `agentpack --json` subprocess) while preserving confirm-token semantics.

#### Scenario: deploy_apply reuses handlers and preserves stable envelopes
- **WHEN** a client calls tool `deploy_apply` with `yes=true` and a valid `confirm_token`
- **THEN** the server applies the deploy plan without spawning an `agentpack --json` subprocess
- **AND** the server returns an Agentpack JSON envelope with `command = "deploy"`
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
