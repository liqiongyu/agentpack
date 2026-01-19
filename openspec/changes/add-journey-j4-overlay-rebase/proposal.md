# Change: Add journey test J4 (overlay sparse/materialize/rebase/deploy)

## Why

Overlay workflows are a core customization path: users create sparse overlays, materialize upstream files for editing, and later rebase overlays as upstream changes.

We need an end-to-end journey test that verifies:
- sparse overlay creation does not copy upstream files by default,
- `--materialize` brings upstream files into the overlay (missing-only),
- `overlay rebase` detects merge conflicts and returns stable `--json` error codes, and
- deploy uses the overlay-composed desired state.

## What Changes

- Add an integration test for Journey J4 using `tests/journeys/common::TestEnv`:
  - create a sparse directory overlay for a base module
  - materialize upstream files into the overlay
  - simulate an upstream update that conflicts with overlay edits and verify `E_OVERLAY_REBASE_CONFLICT`
  - resolve and deploy, asserting the deployed output matches overlay content

## Impact

- Affected specs: none (tests-only)
- Affected code: `tests/` only
- Affected runtime behavior: none
