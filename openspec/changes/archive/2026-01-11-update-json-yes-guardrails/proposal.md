# Change: Require `--yes` for `--json` write commands (stable error code)

## Why
AI-first usage relies on `--json` output as an API. For safety, write commands should not execute silently in `--json` mode unless explicitly confirmed, and missing confirmation should return a stable, machine-readable error code (not a generic unexpected error).

## What Changes
- In `--json` mode, commands that write to disk or mutate git require `--yes`.
- Missing `--yes` returns `ok=false` with error code `E_CONFIRM_REQUIRED`.
- Align existing guardrails (`deploy --apply`, `bootstrap`, `evolve propose`) to use the same stable code.

## Scope
- Applies at least to: `add`, `remove`, `lock`, `fetch`, `remote set`, `sync`, `record`.

## Acceptance
- Missing `--yes` in `--json` mode does not perform writes and returns `E_CONFIRM_REQUIRED`.
- Behavior is covered by tests.
