# Change: Add `reason_code` and `next_actions` to desired-state conflict errors

## Why
Automation can reliably branch on stable error codes like `E_DESIRED_STATE_CONFLICT`, but still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, desired-state conflict errors include useful context (module_ids + hashes on both sides) but do not include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for `E_DESIRED_STATE_CONFLICT`:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- `E_DESIRED_STATE_CONFLICT` in `--json` mode includes `errors[0].details.reason_code` and `errors[0].details.next_actions`.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
