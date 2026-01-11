## 1. Implementation

### 1.1 Warning detection
- [x] Detect if each target root is inside a git repo.
- [x] Check whether `<target_root>/.agentpack.manifest.json` is ignored by git.
- [x] If not ignored, emit an actionable warning (human + JSON warnings).

### 1.2 `doctor --fix`
- [x] Add `--fix` flag to `agentpack doctor`.
- [x] Implement idempotent `.gitignore` update for each detected repo.
- [x] In `--json` without `--yes`, refuse `doctor --fix` with `E_CONFIRM_REQUIRED` when it would write.

### 1.3 Tests + docs
- [x] Add tests covering warning detection and `--fix` idempotency.
- [x] Update `docs/SPEC.md` doctor section.
