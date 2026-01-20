# Change: Refactor deploy apply logic into handlers

## Why

Deploy apply logic is currently duplicated across:
- CLI `deploy --apply`
- TUI apply (`tui_apply`)

This duplication increases maintenance cost and makes it easier for behavior/guardrails to drift over time.

## What Changes

- Add a deploy handler that centralizes the deploy apply pipeline (adopt guardrails, manifest-write behavior, apply execution).
- Refactor CLI `deploy --apply` and `tui_apply` to reuse the handler.
- Keep user-facing behavior and `--json` contracts unchanged.

## Impact

- Affected specs: none (refactor-only; no user-facing behavior change expected)
- Affected code: `src/handlers/*`, `src/cli/commands/deploy.rs`, `src/tui_apply.rs`, `src/target_manifest.rs`
- Affected runtime behavior: none expected (covered by existing CLI/TUI/journey tests)
