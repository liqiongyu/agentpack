## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new guidance detail fields for `E_POLICY_VIOLATIONS`.

### 1.2 Code changes
- [x] Extend `E_POLICY_VIOLATIONS` error `details` to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new `E_POLICY_VIOLATIONS` detail fields.
- [x] Update `docs/reference/error-codes.md` for `E_POLICY_VIOLATIONS`.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on `E_POLICY_VIOLATIONS`.

### 1.5 Validation
- [x] Run `openspec validate add-policy-violations-next-actions --strict --no-interactive`.
- [x] Run `just check`.
