# Change: Add `reason_code` and `next_actions` to policy config errors

## Why
Automation can reliably branch on stable policy error codes like `E_POLICY_CONFIG_MISSING`, but it still needs structured, machine-actionable guidance for remediation (without parsing human strings).

Today, policy config errors include human `hint` strings, but do not consistently include stable `reason_code` + `next_actions` fields that orchestrators can depend on.

## What Changes
- Add additive fields under `errors[0].details` for policy config errors:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Apply to these stable error codes:
  - `E_POLICY_CONFIG_MISSING`
  - `E_POLICY_CONFIG_INVALID`
  - `E_POLICY_CONFIG_UNSUPPORTED_VERSION`
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- Policy config errors in `--json` mode include `errors[0].details.reason_code` and `errors[0].details.next_actions`.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
