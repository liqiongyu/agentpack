## ADDED Requirements

### Requirement: MCP evolve_propose runs in-process
The system SHALL compute and apply MCP tool `evolve_propose` in-process (without spawning an `agentpack --json` subprocess).

#### Scenario: evolve_propose reuses handlers and preserves stable envelopes
- **WHEN** a client calls tool `evolve_propose` in `--json` mode
- **THEN** the server executes evolve propose without spawning an `agentpack --json` subprocess
- **AND** the server returns an Agentpack JSON envelope with `command = "evolve.propose"`
- **AND** on errors, the server returns an envelope with stable `UserError` codes when applicable
