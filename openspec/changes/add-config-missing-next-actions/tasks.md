## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new guidance detail fields for `E_CONFIG_MISSING`.

### 1.2 Code changes
- [x] Extend `E_CONFIG_MISSING` error `details` to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new `E_CONFIG_MISSING` detail fields.
- [x] Update `docs/reference/error-codes.md` for `E_CONFIG_MISSING`.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on `E_CONFIG_MISSING`.

### 1.5 Validation
- [x] Run `openspec validate add-config-missing-next-actions --strict --no-interactive`.
- [x] Run `just check`.
