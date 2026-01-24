# Change: Add `reason_code` and `next_actions` to adopt-related refusal errors

## Why
Automation (and MCP hosts) can reliably branch on stable error codes like `E_ADOPT_CONFIRM_REQUIRED`, but still need structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, adopt-related refusal errors include useful context like `sample_paths`, but do not include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for adopt-related refusals:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Cover this error:
  - `E_ADOPT_CONFIRM_REQUIRED`
- Update `docs/SPEC.md`, `docs/reference/json-api.md`, and `docs/reference/error-codes.md` to document the new additive fields.
- Add/extend tests to assert the new detail fields for both CLI (`--json`) and MCP flows.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- For `E_ADOPT_CONFIRM_REQUIRED`, `errors[0].details.reason_code` and `errors[0].details.next_actions` are present and stable.
- Existing detail fields (e.g. `details.flag`, `details.adopt_updates`, `details.sample_paths`) are preserved.
- Docs and tests are updated accordingly.
