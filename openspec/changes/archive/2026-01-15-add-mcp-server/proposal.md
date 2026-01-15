# Change: add MCP server (stdio) for Agentpack

## Why
Agentpack already provides a stable, machine-consumable CLI (`--json`) for `plan/diff/status/deploy/rollback/doctor`, but agent runtimes increasingly prefer a structured tool registry over raw CLI invocation. Adding an MCP server makes Agentpack easier to consume by Codex/VS Code/other hosts while keeping Agentpack as the single source of truth for behavior and safety.

## What Changes
- Add an MCP server entrypoint (`agentpack mcp serve`) using **stdio** transport.
- Target MCP protocol versions: `2025-06-18` (preferred) and `2025-03-26` (compat).
- Expose a minimal tool set:
  - read-only: `plan`, `diff`, `status`, `doctor`
  - mutating (explicit approval): `deploy_apply`, `rollback`
- Tool results reuse Agentpack’s existing `--json` envelope as the single stable payload.
  - In MCP, results include both `structuredContent` (JSON object) and a `text` block containing the serialized JSON envelope.
- Safety model:
  - Mutating tools REQUIRE explicit approval (`yes=true`) or return `E_CONFIRM_REQUIRED` and perform no writes.
  - The MCP server MUST NOT emit non-protocol output on stdout (logs go to stderr).
  - `docs/SPEC.md` updates should land alongside the implementation PRs (avoid documenting a feature that does not exist yet).

## Impact
- Affected specs:
  - `agentpack-cli` (new command entrypoint)
  - `agentpack-mcp` (new capability)
- Affected code (planned):
  - new MCP server module(s) and CLI wiring
  - integration tests that drive MCP over stdio
- Backwards compatibility:
  - additive-only for existing CLI behavior and existing JSON schemas
  - new command/tools only
