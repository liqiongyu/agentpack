## 1. Implementation

### 1.1 Spec deltas
- [x] Add OpenSpec deltas for `agentpack-cli` documenting the new refusal detail fields for git refusal errors.

### 1.2 Code changes
- [x] Extend git-related refusal errors `details` to include stable `reason_code` + `next_actions` (additive).

### 1.3 Docs
- [x] Update `docs/SPEC.md` to document the new git refusal detail fields.
- [x] Update `docs/reference/error-codes.md` for the affected git errors.

### 1.4 Tests
- [x] Extend CLI tests to assert `reason_code` + `next_actions` on the affected git refusal errors.

### 1.5 Validation
- [x] Run `openspec validate add-git-refusal-next-actions --strict --no-interactive`.
- [x] Run `just check`.
