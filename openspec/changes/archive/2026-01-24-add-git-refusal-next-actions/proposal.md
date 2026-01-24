# Change: Add `reason_code` and `next_actions` to git-related refusal errors

## Why
Automation can reliably branch on stable error codes like `E_GIT_WORKTREE_DIRTY`, but still needs structured, machine-actionable guidance for what to do next (without parsing human strings).

Today, git-related refusal errors include helpful context (`repo`, `remote`, `hint`, etc.) but do not include stable `reason_code` + `next_actions` for orchestrators.

## What Changes
- Add additive fields under `errors[0].details` for git-related refusal errors:
  - `reason_code: string` (stable, enum-like)
  - `next_actions: string[]` (stable, enum-like action identifiers)
- Cover these errors:
  - `E_GIT_REPO_REQUIRED`
  - `E_GIT_WORKTREE_DIRTY`
  - `E_GIT_DETACHED_HEAD`
  - `E_GIT_REMOTE_MISSING`
  - `E_GIT_NOT_FOUND`
- Update `docs/SPEC.md` and `docs/reference/error-codes.md` to document the new additive fields.
- Extend tests to assert the new detail fields.

## Impact
- Backward compatible: additive fields only (no `schema_version` bump).
- Improves orchestrator ergonomics without changing error codes or exit behavior.

## Acceptance
- For each covered git refusal error, `errors[0].details.reason_code` and `errors[0].details.next_actions` are present and stable.
- Existing detail fields are preserved.
- Docs and tests are updated accordingly.
