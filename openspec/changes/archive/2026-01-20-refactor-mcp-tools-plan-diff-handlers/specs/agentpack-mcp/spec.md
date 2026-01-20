## ADDED Requirements

### Requirement: MCP plan/diff run in-process
The system SHALL compute MCP tools `plan` and `diff` in-process (without spawning an `agentpack --json` subprocess) by invoking the same read-only handler logic used by the CLI.

#### Scenario: plan/diff use handlers and preserve stable envelopes
- **WHEN** a client calls tool `plan` or `diff`
- **THEN** the server returns an Agentpack JSON envelope with the expected `command` value
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
