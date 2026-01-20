## ADDED Requirements

### Requirement: MCP deploy runs in-process for planning
The system SHALL compute MCP tool `deploy` (planning stage) in-process (without spawning an `agentpack --json` subprocess) by invoking the same read-only handler logic used by the CLI.

#### Scenario: deploy uses handlers and preserves stable envelopes
- **WHEN** a client calls tool `deploy`
- **THEN** the server returns an Agentpack JSON envelope with `command = "deploy"`
- **AND** the server returns a confirm token bound to the computed plan (`confirm_token` + `confirm_plan_hash`)
