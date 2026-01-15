# MCP (Codex integration)

Agentpack provides an MCP server over stdio:

```bash
agentpack mcp serve
```

This lets MCP-capable clients (including Codex) call Agentpack as structured tools instead of shelling out and parsing text.

## What tools are exposed?

Minimum tool set:
- read-only: `plan`, `diff`, `status`, `doctor`
- mutating (explicit approval): `deploy_apply`, `rollback`

Tool results reuse Agentpack’s stable `--json` envelope as the canonical payload (also returned as serialized JSON text).

Mutating tools are approval-gated:
- `deploy_apply` and `rollback` require `yes=true`, otherwise they return `E_CONFIRM_REQUIRED`.

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
# enabled_tools = ["plan", "diff", "status", "doctor", "deploy_apply", "rollback"]

enabled = true
```

Notes:
- `agentpack mcp serve` does **not** support `--json` (stdout is reserved for MCP protocol messages).
- If `agentpack` is not on Codex’s PATH, set `command` to an absolute path to the `agentpack` binary.

For Codex-side MCP configuration details, see:
- https://developers.openai.com/codex/mcp/

## Common pitfalls

### Wrong working directory (project overlays don’t apply)

Agentpack determines the active project context (and thus project overlays) from the MCP server process working directory.

Fix:
- Set `cwd` to the intended project root in `~/.codex/config.toml`, or
- Start Codex from the project root and avoid overriding `cwd`.

### Wrong Agentpack home / config repo

Agentpack uses `AGENTPACK_HOME` to locate the config repo and snapshots:
- config repo default: `$AGENTPACK_HOME/repo`
- snapshots: `$AGENTPACK_HOME/state/snapshots`

Fix:
- Set `env = { AGENTPACK_HOME = "..." }` in your MCP server entry, or
- Pass the `repo` argument to tools that support it.

### Mutations are refused without explicit approval

Mutating tools require `yes=true`:
- `deploy_apply` → `E_CONFIRM_REQUIRED` unless `yes=true`
- `rollback` → `E_CONFIRM_REQUIRED` unless `yes=true`

Even with approval:
- `deploy_apply` may still refuse with `E_ADOPT_CONFIRM_REQUIRED` unless `adopt=true` and you explicitly want to overwrite unmanaged files.

### Debugging (stdio transport)

If a client reports that the MCP server is “unresponsive”, verify it can start cleanly:

```bash
agentpack mcp serve
```

The process should not print anything except MCP protocol messages to stdout.
