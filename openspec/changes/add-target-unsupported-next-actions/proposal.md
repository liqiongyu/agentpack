# Change: Add `reason_code` and `next_actions` to target-unsupported errors

## Why
Automation can reliably branch on stable error codes like `E_TARGET_UNSUPPORTED`, but still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, target selection/validation errors include useful context (e.g., `allowed`, `compiled`, `missing`), but do not consistently include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for `E_TARGET_UNSUPPORTED`:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- `E_TARGET_UNSUPPORTED` in `--json` mode includes `errors[0].details.reason_code` and `errors[0].details.next_actions`.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
