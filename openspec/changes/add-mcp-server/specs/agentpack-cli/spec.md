# agentpack-cli (delta)

## ADDED Requirements

### Requirement: Provide an MCP server entrypoint
The system SHALL provide an MCP server entrypoint as a CLI command: `agentpack mcp serve`.

The entrypoint SHALL use stdio transport and SHALL NOT support Agentpack `--json` output mode (stdout is reserved for MCP protocol messages).

#### Scenario: mcp serve can be discovered by help
- **WHEN** the user runs `agentpack help --json`
- **THEN** `data.commands[]` includes a command with `path = ["mcp","serve"]`
