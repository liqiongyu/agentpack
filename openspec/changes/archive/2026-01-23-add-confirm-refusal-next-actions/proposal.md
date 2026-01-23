# Change: Add `reason_code` and `next_actions` to confirm-related refusal errors

## Why
Automation (and MCP hosts) can reliably branch on stable error codes like `E_CONFIRM_REQUIRED`, but often still need structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, confirm-related refusal errors primarily include a message and (sometimes) a free-form `hint`, but do not consistently provide stable `reason_code` + `next_actions` fields.

## What Changes
- Add additive fields under `errors[0].details` for confirm-related refusals:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Cover these errors first (small initial slice):
  - `E_CONFIRM_REQUIRED`
  - `E_CONFIRM_TOKEN_REQUIRED`
  - `E_CONFIRM_TOKEN_EXPIRED`
  - `E_CONFIRM_TOKEN_MISMATCH`
- Update `docs/SPEC.md` and `docs/reference/json-api.md` to document the new additive fields.
- Add/extend tests to assert the new detail fields for both CLI (`--json`) and MCP flows.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- For the listed error codes, `errors[0].details.reason_code` and `errors[0].details.next_actions` are present and stable.
- Existing detail fields (e.g. `details.command`, `details.hint`) are preserved.
- Docs and tests are updated accordingly.
