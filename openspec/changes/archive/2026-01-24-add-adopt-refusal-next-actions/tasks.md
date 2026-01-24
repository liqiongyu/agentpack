## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` and `agentpack-mcp` documenting the new refusal detail fields for `E_ADOPT_CONFIRM_REQUIRED`.

### 1.2 Code changes
- [x] Extend the `E_ADOPT_CONFIRM_REQUIRED` error `details` to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new adopt-related refusal detail fields.
- [x] Update `docs/reference/json-api.md` guidance (additive fields).
- [x] Update `docs/reference/error-codes.md` for `E_ADOPT_CONFIRM_REQUIRED` details.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on `E_ADOPT_CONFIRM_REQUIRED`.
- [x] Extend MCP tests to assert `reason_code` + `next_actions` on `E_ADOPT_CONFIRM_REQUIRED`.

### 1.5 Validation
- [x] Run `openspec validate add-adopt-refusal-next-actions --strict --no-interactive`.
- [x] Run `just check`.
