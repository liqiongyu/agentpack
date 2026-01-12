# Change: Adopt confirmation for unmanaged overwrites

## Why
On a first deploy (no prior target manifest and no prior snapshot), Agentpack currently treats any existing file at a desired output path as a normal `update` and will overwrite it when applying. This can silently clobber user-owned, unmanaged files.

## What Changes
- Detect and classify “adopt updates”: updates that would overwrite an existing file that is **not known to be managed**.
- Refuse to apply adopt updates unless the user explicitly opts in (separate from `--yes`).
- In `--json` mode, return a stable error code for missing adopt confirmation so automation can branch reliably.
- Expose adopt updates in plan/preview output (machine-readable) so callers can surface a clear prompt before apply.

## Impact
- Affected specs:
  - `openspec/specs/agentpack/spec.md` (new requirement)
  - `docs/SPEC.md` (CLI + `--json` contract update)
- Affected code:
  - `src/deploy.rs` (plan classification)
  - `src/cli.rs` (deploy flag + enforcement)
- Affected tests:
  - golden snapshot updates under `tests/golden/`
  - new CLI tests for `E_ADOPT_CONFIRM_REQUIRED`
