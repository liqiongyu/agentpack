# Change: Refactor MCP evolve_propose tool to run in-process

## Why

The MCP server still shells out to `agentpack evolve propose --json` for the mutating `evolve_propose` tool. This adds overhead and is the last remaining MCP tool that spawns an `agentpack --json` subprocess, blocking progress toward M3-REF-004 (single-source business logic across CLI/MCP/TUI).

## What Changes

- Add an in-process handler for `evolve propose` and route MCP tool `evolve_propose` through it (no subprocess).
- Preserve the exact Agentpack `--json` envelope shape (`command = "evolve.propose"`) and stable error codes.

## Impact

- Affected specs: none (refactor-only; expected no user-facing behavior change)
- Affected code: `src/handlers/evolve.rs`, `src/cli/commands/evolve.rs`, `src/mcp/tools.rs`
