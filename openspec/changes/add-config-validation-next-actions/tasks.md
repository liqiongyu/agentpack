## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new guidance detail fields for config validation error codes.

### 1.2 Code changes
- [x] Extend config validation errors to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new config validation error `details` fields.
- [x] Update `docs/reference/error-codes.md` for `E_CONFIG_INVALID` and `E_CONFIG_UNSUPPORTED_VERSION`.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on config validation error codes.

### 1.5 Validation
- [x] Run `openspec validate add-config-validation-next-actions --strict --no-interactive`.
- [x] Run `just check`.
