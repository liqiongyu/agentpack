## ADDED Requirements

### Requirement: MCP doctor runs in-process
The system SHALL compute MCP tool `doctor` in-process (without spawning an `agentpack --json` subprocess) by invoking the same handler logic used by the CLI.

#### Scenario: doctor uses handlers and preserves stable envelopes
- **WHEN** a client calls tool `doctor`
- **THEN** the server returns an Agentpack JSON envelope with `command = "doctor"`
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
