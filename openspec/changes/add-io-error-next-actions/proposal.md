# Change: Add `reason_code` and `next_actions` to IO errors

## Why
Automation can reliably branch on stable IO error codes like `E_IO_PERMISSION_DENIED`, but it still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, these stable `E_IO_*` errors include helpful `hint` strings, but do not consistently include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for stable IO write errors:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Apply to these stable error codes:
  - `E_IO_PERMISSION_DENIED`
  - `E_IO_INVALID_PATH`
  - `E_IO_PATH_TOO_LONG`
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend conformance tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- In `--json` mode, IO write failures classified as stable `E_IO_*` codes include `errors[0].details.reason_code` and `errors[0].details.next_actions`.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
