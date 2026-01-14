## 1. Spec (OpenSpec)
- [x] Define `agentpack overlay edit --kind patch` behavior in the delta spec under `openspec/changes/add-patch-overlay-edit/specs/agentpack-cli/spec.md`.

## 2. Docs (implementation contract)
- [x] Update `docs/SPEC.md` overlay edit section to document patch overlay skeleton creation.

## 3. Validation
- [x] `openspec validate add-patch-overlay-edit --strict --no-interactive`

## 4. Implementation (overlay edit)
- [x] Add `--kind dir|patch` (or equivalent) to `agentpack overlay edit`.
- [x] Create patch overlay skeleton: `.agentpack/overlay.json` with `overlay_kind=patch` + `.agentpack/patches/`.
- [x] Add integration tests for skeleton creation and metadata.

## 5. Archive (after deploy)
- [ ] Archive the change via `openspec archive add-patch-overlay-edit --yes` in a separate PR after the implementation is merged.
