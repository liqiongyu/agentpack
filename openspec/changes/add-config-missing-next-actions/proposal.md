# Change: Add `reason_code` and `next_actions` to config-missing errors

## Why
Automation can reliably branch on the stable error code `E_CONFIG_MISSING`, but still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, `E_CONFIG_MISSING` includes a human `hint`, but does not include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for `E_CONFIG_MISSING`:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- `E_CONFIG_MISSING` in `--json` mode includes `errors[0].details.reason_code` and `errors[0].details.next_actions`.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
