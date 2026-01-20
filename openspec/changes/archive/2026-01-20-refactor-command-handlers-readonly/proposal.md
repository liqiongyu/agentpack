# Change: Refactor read-only command logic into handlers

## Why

Read-only commands (`plan`, `diff`, `preview`, and read-only views for TUI) duplicate the same “load engine → render desired state → resolve managed paths → compute plan” pipeline in multiple places. This increases maintenance cost and makes it easier for CLI/TUI to drift in behavior over time.

Introducing a small handlers layer for the shared read-only planning pipeline improves maintainability while preserving user-facing behavior and stable JSON contracts.

## What Changes

- Add `src/handlers/` for shared command business logic.
- Introduce a read-only planning helper (compute desired state + managed paths + plan).
- Refactor CLI commands (`plan`, `diff`, `preview`, `deploy` plan-only path) and TUI read-only views to reuse the handler.

## Impact

- Affected specs: none (no user-facing behavior change)
- Affected code: `src/cli/commands/*`, `src/tui_core.rs`, new `src/handlers/*`
- Affected runtime behavior: none expected
