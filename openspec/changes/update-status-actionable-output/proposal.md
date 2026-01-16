# Change: update status actionable output (grouped summary + structured next_actions)

## Why

`agentpack status` is the first command an agent (or user) should run when something is off. Today, `status --json` includes a flat drift summary and optional `next_actions` as command strings, but agents still have to infer meaning from strings and cannot easily group drift by target root.

We want to make `status` more action-oriented for automation while keeping `schema_version=1` compatibility (additive-only JSON changes).

## What Changes

- Add an additive grouped drift summary in `status --json` (by `(target, root)`).
- Add an additive structured `next_actions` representation in `status --json` that includes a stable `action` code plus a suggested `command`.
- Improve human output to display per-root summaries and recommended follow-up commands.
- Update docs and JSON goldens to lock the new fields.

## Non-Goals

- No breaking changes to the `schema_version=1` JSON contract (do not remove/rename existing fields).
- Do not change drift detection semantics.

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`
- Affected code: `src/cli/commands/status.rs`, `src/cli/commands/schema.rs`
- Affected docs: `docs/JSON_API.md`, `docs/SPEC.md`
- Affected tests: `tests/golden/status_json_data.json`, `tests/golden/schema_json_data.json`
