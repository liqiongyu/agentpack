## 1. Spec (OpenSpec)
- [x] Define patch overlay rebase behavior in the delta spec under `openspec/changes/archive/2026-01-14-add-patch-overlay-rebase/specs/agentpack-cli/spec.md`.

## 2. Docs (implementation contract)
- [x] Update `docs/SPEC.md` overlay rebase section to document patch overlays.

## 3. Validation
- [x] `openspec validate add-patch-overlay-rebase --strict --no-interactive`

## 4. Implementation (patch overlay rebase)
- [x] Detect `overlay_kind=patch` and rebase `.agentpack/patches/<relpath>.patch` files.
- [x] Use the overlay baseline as merge base and rewrite patches against the latest upstream.
- [x] On conflicts: write `.agentpack/conflicts/<relpath>` and return `E_OVERLAY_REBASE_CONFLICT`.
- [x] Delete no-op patches and prune now-empty parent directories under `.agentpack/patches/`.
- [x] Add integration tests for success + conflict paths.

## 5. Archive (after deploy)
- [x] Archive the change via `openspec archive add-patch-overlay-rebase --yes` in a separate PR after the implementation is merged.
