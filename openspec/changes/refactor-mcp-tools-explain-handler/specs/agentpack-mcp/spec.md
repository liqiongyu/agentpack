## ADDED Requirements

### Requirement: MCP explain runs in-process
The system SHALL compute MCP tool `explain` in-process (without spawning an `agentpack --json` subprocess) by invoking the same logic used by the CLI.

#### Scenario: explain uses in-process logic and preserves stable envelopes
- **WHEN** a client calls tool `explain` with `kind=plan|diff|status`
- **THEN** the server returns an Agentpack JSON envelope with the expected `command` value
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
