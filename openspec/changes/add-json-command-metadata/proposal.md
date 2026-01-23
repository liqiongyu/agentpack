# Change: Add `command_id` and `command_path` to the `--json` envelope

## Why
Automation needs to reliably identify the invoked command, including subcommands and mutating variants like `deploy --apply` / `doctor --fix`.

Today the envelope’s `command` field is not sufficient for this:
- Many success envelopes use dotted leaf names (e.g. `overlay.path`, `remote.set`).
- The error envelope currently only includes the top-level command (e.g. `overlay`, `remote`, `evolve`), losing subcommand specificity.

Adding explicit command metadata fields makes `--json` outputs easier to orchestrate without breaking existing parsers.

## What Changes
- Add two new top-level fields to the `schema_version=1` envelope:
  - `command_id: string` (stable, space-separated command id aligned with `help --json` ids and mutating guardrails, e.g. `remote set`, `deploy --apply`)
  - `command_path: string[]` (tokenized `command_id`)
- Populate these fields for both success and error envelopes.
- Update `agentpack schema --json`, `docs/SPEC.md`, and `docs/reference/json-api.md`.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Affects stable API output in `--json` mode; requires updating golden snapshots and docs.

## Acceptance
- All `--json` outputs include `command_id` and `command_path`.
- For subcommands, `command_id` reflects the full leaf id (e.g. `remote set`), even on failure.
- For mutating variants, `command_id` reflects the variant id used by guardrails (e.g. `deploy --apply`, `doctor --fix`, `import --apply`).
