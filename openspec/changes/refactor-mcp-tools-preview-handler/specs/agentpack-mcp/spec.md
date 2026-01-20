## ADDED Requirements

### Requirement: MCP preview runs in-process
The system SHALL compute MCP tool `preview` in-process (without spawning an `agentpack --json` subprocess) by invoking the same handler logic used by the CLI.

#### Scenario: preview uses handlers and preserves stable envelopes
- **WHEN** a client calls tool `preview` (with `diff=true` or `diff=false`)
- **THEN** the server returns an Agentpack JSON envelope with the expected `command` value
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
