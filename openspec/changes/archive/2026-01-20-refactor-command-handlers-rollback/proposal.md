# Change: Refactor rollback command logic into handlers

## Why

As part of M3-REF-004 (shared command handlers), mutating command guardrails should be enforced in the handlers layer so CLI/MCP/TUI callers share a single source of truth.

Rollback is a core mutating command and is a small, contained step toward that goal.

## What Changes

- Add a rollback handler that centralizes rollback execution and `--json` guardrails.
- Refactor CLI `rollback` to delegate to the handler.
- Keep user-facing behavior and `--json` contracts unchanged.

## Impact

- Affected specs: none (refactor-only; no user-facing behavior change expected)
- Affected code: `src/handlers/*`, `src/cli/commands/rollback.rs`
- Affected runtime behavior: none expected (covered by existing CLI/MCP/journey tests)
