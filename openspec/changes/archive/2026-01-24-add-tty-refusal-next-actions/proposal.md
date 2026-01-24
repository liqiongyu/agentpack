# Change: Add `reason_code` and `next_actions` to TTY-related refusal errors

## Why
Automation can reliably branch on stable error codes like `E_TTY_REQUIRED`, but still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, `init --guided --json` in a non-TTY context returns `E_TTY_REQUIRED` with useful context fields, but does not include stable `reason_code` + `next_actions` for orchestrators.

## What Changes
- Add additive fields under `errors[0].details` for `E_TTY_REQUIRED`:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- For `E_TTY_REQUIRED`, `errors[0].details.reason_code` and `errors[0].details.next_actions` are present and stable.
- Existing detail fields (e.g. `stdin_is_terminal`, `stdout_is_terminal`, `hint`) are preserved.
- Docs and tests are updated accordingly.
