## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new refusal detail fields for `E_IMPORT_CONFLICT`.

### 1.2 Code changes
- [x] Extend the `E_IMPORT_CONFLICT` error `details` to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new import-conflict refusal detail fields.
- [x] Update `docs/reference/error-codes.md` for `E_IMPORT_CONFLICT` details.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on `E_IMPORT_CONFLICT`.

### 1.5 Validation
- [x] Run `openspec validate add-import-conflict-next-actions --strict --no-interactive`.
- [x] Run `just check`.
