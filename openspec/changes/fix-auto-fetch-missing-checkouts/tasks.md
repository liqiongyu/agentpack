## 1. Implementation

### 1.1 Auto-fetch missing checkouts
- [x] When using lockfile-resolved git modules, if the checkout dir is missing, call `ensure_git_checkout()`.
- [x] Ensure this triggers only for missing checkouts (no redundant fetches).
- [x] Keep errors actionable on failures.

### 1.2 Tests + docs
- [x] Add a test that simulates a missing checkout dir and verifies it is auto-populated.
- [x] Update `docs/SPEC.md` to document the v0.3 cache-miss behavior.
