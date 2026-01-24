# Change: Add `reason_code` and `next_actions` to overlay conflict errors

## Why
Automation can reliably branch on stable error codes like `E_OVERLAY_REBASE_CONFLICT`, but still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, overlay conflict errors include useful context (conflict file lists, patch stderr, etc.) but do not include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for overlay conflict errors:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Cover these errors:
  - `E_OVERLAY_REBASE_CONFLICT`
  - `E_OVERLAY_PATCH_APPLY_FAILED`
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- For each covered overlay conflict error, `errors[0].details.reason_code` and `errors[0].details.next_actions` are present and stable.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
