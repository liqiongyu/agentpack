## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` and `agentpack-mcp` documenting the new refusal detail fields.

### 1.2 Code changes
- [x] Extend `UserError::confirm_required` to include stable `reason_code` + `next_actions` in `details` (additive).
- [x] Extend `UserError::confirm_token_required|expired|mismatch` to include stable `reason_code` + `next_actions` in `details` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new confirm-related refusal detail fields.
- [x] Update `docs/reference/json-api.md` failure examples and guidance (additive fields).

### 1.4 Tests
- [x] Add/extend CLI tests to assert `reason_code` + `next_actions` on `E_CONFIRM_REQUIRED`.
- [x] Add/extend MCP tests to assert `reason_code` + `next_actions` on `E_CONFIRM_TOKEN_*` and `E_CONFIRM_REQUIRED` where applicable.

### 1.5 Validation
- [x] Run `openspec validate add-confirm-refusal-next-actions --strict --no-interactive`.
- [x] Run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all --locked`.
