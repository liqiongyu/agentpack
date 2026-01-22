## ADDED Requirements

### Requirement: preview JSON payload remains consistent across CLI and MCP
The system SHALL centralize the construction of the `preview` JSON `data` payload so that CLI `preview --json` and the MCP `preview` tool keep equivalent output over time.

#### Scenario: preview JSON payload remains consistent
- **GIVEN** a preview result that yields a plan and optional diffs
- **WHEN** the user runs `agentpack preview --json` (optionally with `--diff`)
- **AND** an MCP client calls the `preview` tool (optionally with `diff=true`)
- **THEN** both responses include equivalent `data` fields (`profile`, `targets`, `plan`, optional `diff`) modulo surface-appropriate flags
