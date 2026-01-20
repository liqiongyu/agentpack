## ADDED Requirements

### Requirement: MCP rollback runs in-process
The system SHALL compute MCP tool `rollback` in-process (without spawning an `agentpack --json` subprocess) by invoking the same rollback handler logic used by the CLI.

#### Scenario: rollback uses handler and preserves stable envelopes
- **WHEN** a client calls tool `rollback`
- **THEN** the server returns an Agentpack JSON envelope with the expected `command` value
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
