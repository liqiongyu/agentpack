# Change: Add `next_actions` to `agentpack status` output

## Why
`agentpack status` already detects drift and operator-asset issues, but agents and humans still have to interpret warnings and decide the next command. Adding a structured `next_actions` list makes the output more actionable without breaking the stable `schema_version=1` JSON contract (additive-only change).

## What Changes
- Add an additive `data.next_actions` field to `agentpack status --json`.
- Improve human `status` output by printing a “Next actions” section when applicable.
- Update `agentpack schema` to document the new `status` data field.

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/cli/commands/status.rs`, `src/cli/commands/schema.rs`
- Affected docs: `docs/SPEC.md`, `docs/JSON_API.md`
- Affected tests: golden snapshots under `tests/golden/`
