## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new guidance detail fields for policy config error codes.

### 1.2 Code changes
- [x] Extend policy config errors to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new policy config error `details` fields.
- [x] Update `docs/reference/error-codes.md` for policy config error codes.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on policy config error codes.

### 1.5 Validation
- [x] Run `openspec validate add-policy-config-next-actions --strict --no-interactive`.
- [x] Run `just check`.
