## 1. Spec (OpenSpec)
- [x] Add patch overlay requirements to `openspec/specs/agentpack/spec.md` and `openspec/specs/agentpack-cli/spec.md` (delta).

## 2. Docs (implementation contract)
- [x] Update `docs/SPEC.md` overlays section to define patch overlay layout and precedence rules.

## 3. Validation
- [x] `openspec validate add-patch-overlays --strict --no-interactive`

## 4. Implementation (patch apply)
- [x] Apply `.agentpack/patches/<relpath>.patch` during desired-state generation when `overlay_kind=patch`.
- [x] Reject binary/non-UTF8 patch overlays (text-only).
- [x] Return stable error code `E_OVERLAY_PATCH_APPLY_FAILED` when patch application fails.
- [x] Add integration tests for patch overlay application + failure mode.

## 5. Archive (after deploy)
- [x] Archive the change via `openspec archive add-patch-overlays --yes` in a separate PR after the implementation is merged.
