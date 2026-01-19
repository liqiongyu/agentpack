# Change: Add journey test J5 (patch overlay generate/rebase/apply)

## Why

Patch overlays are a key customization path for “small edits without copying the whole upstream tree”.

We need an end-to-end journey test that validates:
- creating patch overlays via `overlay edit --kind patch`,
- applying patches during desired-state generation (`deploy --apply` / `plan`),
- rebasing after upstream updates, including conflict detection and discoverable conflict artifacts, and
- stable `--json` error codes for automation.

## What Changes

- Add an integration test for Journey J5 using `tests/journeys/common::TestEnv`:
  - create a patch overlay for a base module
  - add a unified-diff patch under `.agentpack/patches/`
  - deploy and assert patched output
  - simulate an upstream update that conflicts and verify `E_OVERLAY_REBASE_CONFLICT` plus on-disk conflict artifacts

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
