## ADDED Requirements

### Requirement: status operator asset warnings remain consistent across CLI and MCP
The system SHALL centralize the operator-assets warning logic used by CLI `status` and the MCP `status` tool so that they check the same asset locations and keep warning/suggestion behavior consistent over time.

#### Scenario: operator assets warnings remain consistent
- **GIVEN** a repo with a target that uses operator assets (e.g. `codex` or `claude_code`)
- **AND** the installed operator assets are missing or outdated
- **WHEN** the user runs `agentpack status --json`
- **AND** an MCP client calls the `status` tool
- **THEN** both responses include equivalent warnings describing the missing/outdated operator assets
- **AND** both provide equivalent suggested bootstrap commands (with surface-appropriate flags)
