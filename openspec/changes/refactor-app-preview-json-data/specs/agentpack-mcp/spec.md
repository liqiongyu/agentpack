## ADDED Requirements

### Requirement: preview JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `preview` JSON `data` payload so that the MCP `preview` tool stays consistent with CLI `preview --json` over time.

#### Scenario: preview JSON payload remains consistent
- **GIVEN** a preview result that yields a plan and optional diffs
- **WHEN** an MCP client calls the `preview` tool (optionally with `diff=true`)
- **AND** a user runs `agentpack preview --json` (optionally with `--diff`)
- **THEN** both responses include equivalent `data` fields (`profile`, `targets`, `plan`, optional `diff`) modulo surface-appropriate flags
