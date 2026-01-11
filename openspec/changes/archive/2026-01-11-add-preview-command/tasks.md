## 1. Implementation

### 1.1 CLI + behavior
- [x] Add `agentpack preview [--diff]`.
- [x] Always compute plan; when `--diff`, include diffs in human mode.

### 1.2 JSON output
- [x] In `--json`, output `data.plan` and (when `--diff`) `data.diff`.

### 1.3 Tests + docs
- [x] Add integration tests for `preview --json` and `preview --diff --json`.
- [x] Update `docs/SPEC.md` to document `agentpack preview`.
