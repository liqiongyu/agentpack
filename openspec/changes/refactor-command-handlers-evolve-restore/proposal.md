# Change: Refactor evolve.restore logic into handlers

## Why

`evolve restore` is a mutating command with `--json` guardrails and file-write behavior. Keeping its core logic inside the CLI command module makes it harder to share across future callers (MCP/TUI) and increases drift risk.

## What Changes

- Add a handler for `evolve.restore` that centralizes: missing-output detection, confirmation guardrails, and write execution.
- Refactor CLI `evolve restore` to delegate to the handler.
- Preserve user-facing behavior and stable `--json` envelope/error codes.

## Impact

- Affected specs: none (refactor-only; no user-facing behavior change expected)
- Affected code: `src/handlers/*`, `src/cli/commands/evolve.rs`
