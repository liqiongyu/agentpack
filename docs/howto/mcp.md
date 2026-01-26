# MCP (Codex + VS Code integration)

> Language: English | [Chinese (Simplified)](../zh-CN/howto/mcp.md)

Agentpack provides an MCP server over stdio:

```bash
agentpack mcp serve
```

This lets MCP-capable clients (including Codex) call Agentpack as structured tools instead of shelling out and parsing text.

## What tools are exposed?

Tool set:
- read-only: `plan`, `diff`, `preview`, `status`, `doctor`, `deploy`, `explain`
- mutating (explicit approval): `deploy_apply`, `rollback`, `evolve_propose`, `evolve_restore`

Tool results reuse Agentpack’s stable `--json` envelope as the canonical payload (also returned as serialized JSON text).

Mutating tools are approval-gated:
- `deploy_apply`, `rollback`, `evolve_propose`, and `evolve_restore` require `yes=true`, otherwise they return `E_CONFIRM_REQUIRED`.

Two-stage deploy confirmation:
- Call `deploy` first to obtain `data.confirm_token` (and metadata like `data.confirm_plan_hash`, `data.confirm_token_expires_at`).
- Then call `deploy_apply` with `yes=true` and `confirm_token`.
- If the token is missing/expired/mismatched, `deploy_apply` returns `E_CONFIRM_TOKEN_REQUIRED` / `E_CONFIRM_TOKEN_EXPIRED` / `E_CONFIRM_TOKEN_MISMATCH`.

## Codex configuration

Codex MCP servers are configured in `~/.codex/config.toml` under a `[mcp_servers.<name>]` table.

Add an `agentpack` server entry:

```toml
[mcp_servers.agentpack]
command = "agentpack"
args = ["mcp", "serve"]

# Strongly recommended: set the working directory to your project root.
# Agentpack detects project overlays from the process CWD.
cwd = "/path/to/your/project"

# Optional: if you use a non-default Agentpack home.
# env = { AGENTPACK_HOME = "/path/to/.agentpack" }

# Optional: limit which tools Codex can call.
# enabled_tools = [
#   "plan", "diff", "preview", "status", "doctor", "deploy", "explain",
#   "deploy_apply", "rollback", "evolve_propose", "evolve_restore"
# ]

enabled = true
```

Notes:
- `agentpack mcp serve` does **not** support `--json` (stdout is reserved for MCP protocol messages).
- If `agentpack` is not on Codex’s PATH, set `command` to an absolute path to the `agentpack` binary.

For Codex-side MCP configuration details, see:
- https://developers.openai.com/codex/mcp/

## VS Code configuration

VS Code MCP servers can be configured globally (user profile `mcp.json`) or per-workspace (`.vscode/mcp.json`).

This example shows a workspace-scoped `.vscode/mcp.json` for Agentpack:

```json
{
  "servers": {
    "agentpack": {
      "type": "stdio",
      "command": "agentpack",
      "args": ["mcp", "serve"],
      "env": {
        "AGENTPACK_HOME": "${env:AGENTPACK_HOME}"
      }
    }
  }
}
```

Security note:
- Avoid hardcoding sensitive values in `mcp.json`. VS Code supports `inputs` (prompted secrets) and `envFile` for environment-based configuration.
- See: https://code.visualstudio.com/docs/copilot/customization/mcp-servers

## Common pitfalls

### Wrong working directory (project overlays don’t apply)

Agentpack determines the active project context (and thus project overlays) from the MCP server process working directory.

Fix:
- Set `cwd` to the intended project root in `~/.codex/config.toml`, or
- Start Codex from the project root and avoid overriding `cwd`.
- In VS Code, prefer a workspace-scoped `.vscode/mcp.json` in the project root (and open that folder/workspace), so the MCP server inherits the correct working directory.

### Wrong Agentpack home / config repo

Agentpack uses `AGENTPACK_HOME` to locate the config repo and snapshots:
- config repo default: `$AGENTPACK_HOME/repo`
- snapshots: `$AGENTPACK_HOME/state/snapshots`

Fix:
- Set `env = { AGENTPACK_HOME = "..." }` in your MCP server entry, or
- Pass the `repo` argument to tools that support it.

### Mutations are refused without explicit approval

Mutating tools require `yes=true`:
- `deploy_apply` → `E_CONFIRM_REQUIRED` unless `yes=true` (and also requires a matching `confirm_token`)
- `rollback` → `E_CONFIRM_REQUIRED` unless `yes=true`

Even with approval:
- `deploy_apply` may still refuse with `E_ADOPT_CONFIRM_REQUIRED` unless `adopt=true` and you explicitly want to overwrite unmanaged files.

### Debugging (stdio transport)

If a client reports that the MCP server is “unresponsive”, verify it can start cleanly:

```bash
agentpack mcp serve
```

The process should not print anything except MCP protocol messages to stdout.
