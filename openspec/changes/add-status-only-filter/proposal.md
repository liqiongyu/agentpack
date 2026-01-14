# Change: status --only drift filter

## Why
For heavy users, `agentpack status` drift can be large, and it’s common to want to focus on a subset quickly (e.g., only `missing` items).

This change adds a small, script-friendly filter without changing existing default behavior.

## What Changes
- Add `agentpack status --only <missing|modified|extra>` (repeatable or comma-separated) to filter drift items.
- In `--json` mode:
  - keep `data.drift` but filter it when `--only` is set
  - keep `data.summary` but have it reflect the filtered view
  - add `data.summary_total` when filtering is used (additive-only)
- Update docs and tests to lock the contract down.

## Impact
- Affected docs/specs: `docs/SPEC.md`, `docs/CLI.md` (+ zh-CN), `docs/JSON_API.md`, `openspec/specs/agentpack-cli/spec.md`.
- Affected code: `src/cli/args.rs`, `src/cli/dispatch.rs`, `src/cli/commands/status.rs`, `src/cli/commands/schema.rs`.
- Backward compatibility:
  - no behavior change when `--only` is not provided
  - additive-only JSON changes (`summary_total` only appears when filtering is used)
