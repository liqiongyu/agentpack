## 1. Implementation

### 1.1 CLI surface
- [x] Add `agentpack update` command with flags: `--lock`, `--fetch`, `--no-lock`, `--no-fetch` (with sensible conflicts).
- [x] Implement default step selection based on lockfile existence.

### 1.2 JSON output + guardrails
- [x] In `--json`, output aggregated `data.steps` with per-step details.
- [x] In `--json` without `--yes`, refuse when `update` will run any write step, returning `E_CONFIRM_REQUIRED`.

### 1.3 Tests + docs
- [x] Add tests for default behavior and `--json` guardrails.
- [x] Update `docs/SPEC.md` to describe `agentpack update`.
