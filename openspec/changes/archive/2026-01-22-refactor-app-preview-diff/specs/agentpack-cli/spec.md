## ADDED Requirements

### Requirement: Preview diff file generation is shared across interfaces
The system SHALL centralize preview per-file diff payload generation so that `agentpack preview --json --diff` and the MCP `preview` tool (`diff=true`) remain consistent over time.

#### Scenario: CLI preview and MCP preview remain consistent
- **GIVEN** the same repo/profile/target inputs
- **WHEN** the user runs `agentpack preview --json --diff`
- **AND** an MCP client calls the `preview` tool with `diff=true`
- **THEN** both responses include `data.diff.files` with the same schema and equivalent entries
