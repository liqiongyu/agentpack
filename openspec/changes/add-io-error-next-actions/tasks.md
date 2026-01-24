## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting guidance detail fields for stable `E_IO_*` errors.

### 1.2 Code changes
- [x] Ensure stable `E_IO_*` errors include additive `reason_code` + `next_actions` (no behavior changes beyond JSON details).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new IO error `details` fields.
- [x] Update `docs/reference/error-codes.md` for `E_IO_PERMISSION_DENIED`, `E_IO_INVALID_PATH`, and `E_IO_PATH_TOO_LONG`.

### 1.4 Tests
- [x] Extend tests to assert `reason_code` + `next_actions` on stable `E_IO_*` errors.

### 1.5 Validation
- [x] Run `openspec validate add-io-error-next-actions --strict --no-interactive`.
- [x] Run `just check`.
