## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new guidance detail fields for lockfile error codes.

### 1.2 Code changes
- [x] Extend lockfile-related errors to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new lockfile error `details` fields.
- [x] Update `docs/reference/error-codes.md` for lockfile error codes.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on lockfile error codes.

### 1.5 Validation
- [x] Run `openspec validate add-lockfile-next-actions --strict --no-interactive`.
- [x] Run `just check`.
