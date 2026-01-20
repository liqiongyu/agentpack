## ADDED Requirements

### Requirement: MCP evolve_restore runs in-process
The system SHALL compute MCP tool `evolve_restore` in-process (without spawning an `agentpack --json` subprocess) by invoking the same evolve restore handler logic used by the CLI.

#### Scenario: evolve_restore uses handler and preserves stable envelopes
- **WHEN** a client calls tool `evolve_restore`
- **THEN** the server returns an Agentpack JSON envelope with the expected `command` value
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
