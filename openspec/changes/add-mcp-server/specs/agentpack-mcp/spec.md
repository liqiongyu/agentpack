# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: Provide an MCP server over stdio
The system SHALL provide an MCP server entrypoint that communicates over stdio using newline-delimited JSON-RPC messages and advertises the `tools` capability.

#### Scenario: Client can initialize the server
- **WHEN** a client sends an MCP `initialize` request
- **THEN** the server responds with a supported `protocolVersion`
- **AND** `capabilities.tools` is present
- **AND** `serverInfo.name` is `agentpack`

### Requirement: Expose Agentpack operations as MCP tools
The server SHALL expose these tools at minimum:
`plan`, `diff`, `status`, `doctor`, `deploy_apply`, `rollback`.

Each tool’s `inputSchema` SHALL be valid JSON Schema and SHALL include options equivalent to Agentpack CLI flags where applicable (`repo`, `profile`, `target`, `machine`, `dry_run`, `yes`, etc).

#### Scenario: tools/list includes the minimal tool set
- **WHEN** a client calls `tools/list`
- **THEN** the returned tool list includes each of the required tools by name

### Requirement: Tool results reuse the Agentpack JSON envelope
For each tool, the tool result SHALL reuse Agentpack’s stable `--json` envelope as the single canonical payload:
- In MCP `structuredContent`, the server SHALL return the envelope as a JSON object.
- In MCP `content`, the server SHOULD include a `text` block containing the serialized JSON envelope.

On errors, the server SHALL set MCP `isError=true` and SHALL include an envelope with `ok=false` and stable Agentpack error codes when applicable.

#### Scenario: plan tool returns an Agentpack envelope
- **WHEN** a client calls tool `plan` with default inputs
- **THEN** the tool result `structuredContent` includes `schema_version`, `ok`, `command`, `version`

### Requirement: Mutating tools require explicit approval
Mutating tools (`deploy_apply`, `rollback`) SHALL require explicit approval via an input parameter `yes=true`.
If approval is missing, the server MUST return `E_CONFIRM_REQUIRED` and MUST NOT perform writes.

#### Scenario: deploy_apply without yes is refused
- **WHEN** a client calls tool `deploy_apply` without `yes=true`
- **THEN** the tool result has `isError=true`
- **AND** the Agentpack envelope includes `errors[0].code = E_CONFIRM_REQUIRED`

### Requirement: No non-protocol output on stdout
The MCP server MUST NOT write non-protocol output to stdout (to avoid corrupting the MCP transport). Any logs or diagnostics MUST be written to stderr.

#### Scenario: Server does not corrupt stdio transport
- **WHEN** the server emits logs during tool execution
- **THEN** stdout contains only JSON-RPC messages
