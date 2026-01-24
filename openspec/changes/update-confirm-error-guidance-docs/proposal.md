# Change: Document confirm-related guidance fields in error code registry

## Why
The implementation already includes additive `errors[0].details.reason_code` and `errors[0].details.next_actions` for confirm-related stable errors (e.g. `E_CONFIRM_REQUIRED` and MCP `E_CONFIRM_TOKEN_*`).

However, `docs/reference/error-codes.md` does not currently state that these codes include the guidance fields, and a CLI test that exercises `E_CONFIRM_REQUIRED` does not assert the presence of the fields. This creates avoidable uncertainty for automation users and weakens regression protection.

## What Changes
- Update `docs/reference/error-codes.md` to document that confirm-related stable error codes include additive guidance fields `{reason_code, next_actions}`.
- Extend a CLI test to assert `errors[0].details.reason_code` and `errors[0].details.next_actions` for `E_CONFIRM_REQUIRED`.

## Impact
- Backward compatible: documentation + test coverage only; no runtime behavior changes.

## Acceptance
- `docs/reference/error-codes.md` mentions guidance fields for `E_CONFIRM_REQUIRED` and MCP `E_CONFIRM_TOKEN_*` errors.
- CLI test coverage asserts `E_CONFIRM_REQUIRED` includes `details.reason_code` and `details.next_actions`.
