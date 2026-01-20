# Change: Refactor status drift logic into handlers

## Why

`status` contains non-trivial drift detection logic and is implemented in multiple places (CLI and TUI). Centralizing the core drift computation in `src/handlers/` reduces duplication and makes it easier to reuse in future callers while preserving stable CLI/JSON behavior.

## What Changes

- Add a handler module for status drift computation (manifests reading, drift detection, warnings).
- Refactor CLI `status` to delegate drift computation to the handler (preserve `--json` envelope and stable fields).
- Refactor TUI status rendering to reuse the same handler logic.

## Impact

- Affected specs: none (refactor-only; no user-facing behavior change expected)
- Affected code: `src/handlers/*`, `src/cli/commands/status.rs`, `src/tui_core.rs`
