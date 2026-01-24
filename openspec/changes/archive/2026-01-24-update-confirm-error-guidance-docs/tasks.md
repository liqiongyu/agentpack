## 1. Implementation

### 1.1 Docs
- [x] Update `docs/reference/error-codes.md` to document guidance fields for confirm-related errors.

### 1.2 Tests
- [x] Extend an existing CLI test to assert `E_CONFIRM_REQUIRED` includes `details.reason_code` and `details.next_actions`.

### 1.3 Validation
- [x] Run `openspec validate update-confirm-error-guidance-docs --strict --no-interactive`.
- [x] Run `just check`.
