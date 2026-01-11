## 1. Implementation

### 1.1 CLI + overlay scope mapping
- [x] Add `--scope` to `agentpack overlay edit` with values `global|machine|project` (default global).
- [x] Keep `--project` as deprecated alias for scope=project.
- [x] Ensure overlay skeleton behavior remains: copy upstream into overlay dir if missing; write baseline if missing.
- [x] JSON output includes scope + overlay_dir (+ machine_id / project_id).

### 1.2 Tests + docs
- [x] Add tests for overlay skeleton creation across scopes.
- [x] Update `docs/SPEC.md` overlay edit section to document `--scope`.
