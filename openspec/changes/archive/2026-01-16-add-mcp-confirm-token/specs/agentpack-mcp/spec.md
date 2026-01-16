# agentpack-mcp (delta)

## MODIFIED Requirements

### Requirement: Expose Agentpack operations as MCP tools

The server SHALL expose these tools at minimum:
`plan`, `diff`, `preview`, `status`, `doctor`, `deploy`, `deploy_apply`, `rollback`, `evolve_propose`, `evolve_restore`, `explain`.

Each tool’s `inputSchema` SHALL be valid JSON Schema.

Tool input schemas (v1.0):

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

#### Tool: preview
```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "diff": { "type": "boolean", "default": false, "description": "Include diffs (like `agentpack preview --diff`)." },
    "dry_run": { "type": "boolean", "default": false }
  }
}
```

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

#### Tool: deploy
Same as `plan`.

#### Tool: deploy_apply
```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["yes", "confirm_token"],
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "adopt": { "type": "boolean", "default": false, "description": "Allow overwriting existing unmanaged files (adopt updates)." },
    "dry_run": { "type": "boolean", "default": false, "description": "Force dry-run behavior (do not apply even if this tool is the apply variant)." },
    "confirm_token": { "type": "string", "description": "Token returned by the deploy tool; binds the apply to the reviewed plan." },
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

#### Tool: evolve_propose
```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "module_id": { "type": "string", "description": "Only propose changes for a single module id." },
    "scope": { "type": "string", "default": "global", "enum": ["global", "machine", "project"], "description": "Overlay scope to write into." },
    "branch": { "type": "string", "description": "Branch name to create (default: evolve/propose-<timestamp>)." },
    "dry_run": { "type": "boolean", "default": false },
    "yes": { "type": "boolean", "default": false, "description": "Required explicit approval when not dry_run." }
  }
}
```

#### Tool: evolve_restore
```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "module_id": { "type": "string", "description": "Only restore missing outputs attributable to a module id." },
    "dry_run": { "type": "boolean", "default": false },
    "yes": { "type": "boolean", "default": false, "description": "Required explicit approval when not dry_run." }
  }
}
```

#### Tool: explain
```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["kind"],
  "properties": {
    "repo": { "type": "string", "description": "Path to the agentpack config repo (default: $AGENTPACK_HOME/repo)." },
    "profile": { "type": "string", "default": "default" },
    "target": { "type": "string", "default": "all", "enum": ["all", "codex", "claude_code", "cursor", "vscode"] },
    "machine": { "type": "string", "description": "Machine id for machine overlays. Omit to auto-detect." },
    "kind": { "type": "string", "enum": ["plan", "diff", "status"], "description": "Maps to `agentpack explain <kind> --json`." }
  }
}
```

#### Scenario: tools/list includes the stabilized tool set
- **WHEN** a client calls `tools/list`
- **THEN** the returned tool list includes each of the required tools by name

## ADDED Requirements

### Requirement: Deploy uses a two-stage confirmation token for apply
When a client calls the `deploy` tool, the server SHALL return the normal Agentpack `deploy --json` envelope and SHALL include a `data.confirm_token` field.
The server SHOULD also include `data.confirm_plan_hash` and `data.confirm_token_expires_at` to help hosts render a confirmation UI without additional parsing.

The server SHALL compute a `confirm_plan_hash` (SHA-256, hex-encoded) from the reviewed deploy plan and bind the issued token to that hash.

If the client does not supply `yes=true` on `deploy_apply`, the server MUST return `E_CONFIRM_REQUIRED` and MUST NOT perform token validation or any writes.

When a client calls `deploy_apply` in a way that may write (i.e., `yes=true` and not forced `dry_run=true`), the server SHALL require a `confirm_token` input parameter and SHALL refuse to run when:
- the token is missing
- the token is expired
- the token does not match the current deploy plan (plan hash mismatch)

On `deploy_apply`, the server SHALL recompute the current deploy plan hash immediately before applying and MUST refuse if it differs from the hash bound to the token.

The server SHALL treat tokens as short-lived (recommended: <= 10 minutes).

On refusal, the server SHALL return stable error codes:
- `E_CONFIRM_TOKEN_REQUIRED`
- `E_CONFIRM_TOKEN_EXPIRED`
- `E_CONFIRM_TOKEN_MISMATCH`

#### Scenario: deploy tool returns a confirm_token
- **WHEN** a client calls tool `deploy`
- **THEN** the Agentpack envelope includes `data.confirm_token`

#### Scenario: deploy_apply with mismatched token is refused
- **WHEN** a client calls tool `deploy_apply` with `yes=true` and a token that does not match the current deploy plan
- **THEN** the tool result has `isError=true`
- **AND** the Agentpack envelope includes `errors[0].code = E_CONFIRM_TOKEN_MISMATCH`
