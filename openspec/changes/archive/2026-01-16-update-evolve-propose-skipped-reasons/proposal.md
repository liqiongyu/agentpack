# Change: evolve.propose skipped reasons become structured (reason_code + next_actions)

## Why

`agentpack evolve propose --json` currently reports skipped drift items with a string `reason` plus optional free-form `suggestions`. Automation has to interpret strings, and there is no stable per-item `next_actions` list that an agent can execute safely.

We want to make skipped drift reporting more action-oriented and machine-friendly by introducing stable, enum-like reason codes, human-readable messages, and explicit follow-up commands.

## What Changes

- Add additive fields to each `data.skipped[]` item in `evolve.propose --json`:
  - `reason_code` (stable enum-like string)
  - `reason_message` (human readable)
  - `next_actions[]` (suggested command strings; may be empty)
- Keep existing fields (`reason`, `suggestions`, etc.) for `schema_version=1` compatibility.
- Update docs and tests to lock the new fields.

## Non-Goals

- No breaking changes to `schema_version=1` (no field removals/renames).
- Do not change which drift is considered proposeable vs skipped (only reporting).

## Impact

- Affected specs: `openspec/specs/agentpack/spec.md`
- Affected code: `src/cli/commands/evolve.rs`
- Affected docs: `docs/JSON_API.md`, `docs/SPEC.md`
- Affected tests: `tests/cli_evolve_propose_skipped.rs`
