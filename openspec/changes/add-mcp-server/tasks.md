## 1. Contract (P2-4-0)
- [x] Define MCP tool list and naming (`plan`, `diff`, `status`, `doctor`, `deploy_apply`, `rollback`)
- [x] Define JSON Schemas for each tool input (repo/profile/target/machine/dry_run/yes/etc)
- [x] Define tool result mapping: reuse Agentpack `--json` envelope as `structuredContent` + `text`
- [x] Define mutating approval semantics (`yes=true` required; otherwise `E_CONFIRM_REQUIRED`)
- [x] Run `openspec validate add-mcp-server --strict --no-interactive`

## 2. Implementation: server skeleton (P2-4-1)
- [x] Add `agentpack mcp serve` entrypoint (stdio JSON-RPC loop, no stdout noise)
- [x] Implement MCP `initialize` + `initialized` handling with `tools` capability
- [x] Implement `tools/list` for the declared tools
- [x] Implement `tools/call` dispatcher (stubbed responses initially)
- [x] Add integration test: spawn server, run initialize + tools/list + one tools/call

## 3. Implementation: read-only tools (P2-4-2)
- [x] Implement `plan` tool by routing to existing plan engine/CLI JSON output
- [x] Implement `diff` tool by routing to existing diff engine/CLI JSON output
- [x] Implement `status` tool by routing to existing status engine/CLI JSON output
- [x] Implement `doctor` tool by routing to existing doctor engine/CLI JSON output (no `--fix`)

## 4. Implementation: mutating tools (P2-4-3)
- [ ] Implement `deploy_apply` tool (maps to `deploy --apply`; supports `adopt`; requires `yes=true`)
- [ ] Implement `rollback` tool (maps to `rollback --to`; requires `yes=true`)
- [ ] Ensure missing approval returns `E_CONFIRM_REQUIRED` and does not write

## 5. Docs (P2-4-4)
- [ ] Add `docs/MCP.md` with Codex configuration examples and common pitfalls
- [ ] Update `docs/SPEC.md` to document MCP server entrypoint + tool contract
- [ ] Link `docs/MCP.md` from README / Quickstart (advanced usage)

## 6. Archive
- [ ] After shipping: `openspec archive add-mcp-server --yes`
- [ ] Run `openspec validate --all --strict --no-interactive`
