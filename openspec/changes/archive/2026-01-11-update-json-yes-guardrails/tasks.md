## 1. Implementation

### 1.1 Stable error code for confirmation
- [x] Add a typed CLI error that preserves a stable `code` in `--json` mode.
- [x] Update the `--json` error envelope to emit the typed error code (defaulting to `E_UNEXPECTED`).

### 1.2 Guardrails for write commands
- [x] Require `--yes` for `--json` mode on: add/remove/lock/fetch/remote set/sync/record.
- [x] Update existing write guardrails (deploy/apply, bootstrap, evolve propose) to use `E_CONFIRM_REQUIRED`.

### 1.3 Tests + docs
- [x] Add integration tests that assert `E_CONFIRM_REQUIRED` on missing `--yes`.
- [x] Update `docs/SPEC.md` to document the rule and the error code.
