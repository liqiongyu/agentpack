# agentpack-mcp (delta)

## ADDED Requirements

### Requirement: Provide an MCP server over stdio
The system SHALL provide an MCP server entrypoint that communicates over stdio using newline-delimited JSON-RPC messages and advertises the `tools` capability.

The server SHALL support MCP `protocolVersion` values `2025-11-25` and `2025-03-26`. If the client requests a supported version, the server SHALL respond using that same version; otherwise, the server SHALL respond with `protocolVersion = 2025-11-25`.

#### Scenario: Client can initialize the server
- **WHEN** a client sends an MCP `initialize` request
- **THEN** the server responds with a supported `protocolVersion`
- **AND** `capabilities.tools` is present
- **AND** `serverInfo.name` is `agentpack`

### Requirement: Expose Agentpack operations as MCP tools
The server SHALL expose these tools at minimum:
`plan`, `diff`, `status`, `doctor`, `deploy_apply`, `rollback`.

Each tool’s `inputSchema` SHALL be valid JSON Schema.

Tool input schemas (v0.1):

#### Tool: plan
```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "dry_run": { "type": "boolean", "default": false }
  }
}
```

#### Tool: diff
Same as `plan`.

#### Tool: status
```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "only": { "type": "array", "items": { "type": "string", "enum": ["missing", "modified", "extra"] } },
    "dry_run": { "type": "boolean", "default": false }
  }
}
```

#### Tool: doctor
```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] }
  }
}
```

#### Tool: deploy_apply
```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["yes"],
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "adopt": { "type": "boolean", "default": false, "description": "Allow overwriting existing unmanaged files (adopt updates)." },
    "dry_run": { "type": "boolean", "default": false, "description": "Force dry-run behavior (do not apply even if this tool is the apply variant)." },
    "yes": { "const": true, "description": "Required explicit approval for mutating operations." }
  }
}
```

#### Tool: rollback
```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["to", "yes"],
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "to": { "type": "string", "description": "Snapshot id to rollback to." },
    "yes": { "const": true, "description": "Required explicit approval for mutating operations." }
  }
}
```

#### Scenario: tools/list includes the minimal tool set
- **WHEN** a client calls `tools/list`
- **THEN** the returned tool list includes each of the required tools by name

### Requirement: Tool results reuse the Agentpack JSON envelope
For each tool, the tool result SHALL reuse Agentpack’s stable `--json` envelope as the single canonical payload:
- In MCP `structuredContent`, the server SHALL return the envelope as a JSON object.
- In MCP `content`, the server SHOULD include a `text` block containing the serialized JSON envelope.

The envelope `command` field SHALL match the underlying Agentpack CLI command:
- `plan` -> `command = "plan"`
- `diff` -> `command = "diff"`
- `status` -> `command = "status"`
- `doctor` -> `command = "doctor"`
- `deploy_apply` -> `command = "deploy"`
- `rollback` -> `command = "rollback"`

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
