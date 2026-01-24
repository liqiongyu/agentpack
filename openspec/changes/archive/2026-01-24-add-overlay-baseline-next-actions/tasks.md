## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new guidance detail fields for overlay baseline error codes.

### 1.2 Code changes
- [x] Extend overlay baseline errors to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new overlay baseline error `details` fields.
- [x] Update `docs/reference/error-codes.md` for `E_OVERLAY_NOT_FOUND`, `E_OVERLAY_BASELINE_MISSING`, and `E_OVERLAY_BASELINE_UNSUPPORTED`.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on overlay baseline error codes.

### 1.5 Validation
- [x] Run `openspec validate add-overlay-baseline-next-actions --strict --no-interactive`.
- [x] Run `just check`.
