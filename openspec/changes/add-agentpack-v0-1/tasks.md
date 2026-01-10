## 1. Implementation (v0.1)

### 1.1 Core model & repo/store/state layout
- [ ] Implement `AGENTPACK_HOME` resolution (macOS/Linux/Windows) and directory layout (`repo/`, `store/`, `state/`, `logs/`).
- [ ] Implement config repo loader for `agentpack.yaml` (version/profiles/targets/modules validation).
- [ ] Implement project context detection (cwd, git root, git remote) and stable `project_id`.

### 1.2 Sources, lockfile, and store
- [ ] Parse and normalize sources: `local_path` and `git` (url/ref/subdir/shallow).
- [ ] Resolve git refs to a concrete commit and compute deterministic file manifests + sha256.
- [ ] Generate `agentpack.lock.json` with stable ordering and reproducible content.
- [ ] Implement store fetch/cache and sha256 verification.

### 1.3 Overlays
- [ ] Implement overlay resolution and composition:
  - upstream -> `repo/overlays/<module_id>/...` -> `repo/projects/<project_id>/overlays/<module_id>/...`
- [ ] Implement overlay drift warnings (record upstream baseline for edited files).
- [ ] Implement `agentpack overlay edit <module_id> [--project]` (skeleton + editor).

### 1.4 Deploy pipeline
- [ ] Implement plan generation (create/update/delete) for each target with reasons.
- [ ] Implement diff (text + JSON summary).
- [ ] Implement apply with backups and atomic writes (temp -> rename).
- [ ] Implement validate (minimal structural checks for each module type).
- [ ] Implement state snapshots (`deployments/<id>.json`) and rollback by snapshot id.
- [ ] Implement `status` drift detection (expected vs actual).

### 1.5 Target adapters
- [ ] Codex adapter: skills (repo + user), prompts (user), AGENTS (global + repo).
- [ ] Claude Code adapter (files mode): commands (repo + user) + frontmatter validation.

### 1.6 AI-first bootstrap
- [ ] Embed operator assets templates.
- [ ] Implement `bootstrap` to install Codex operator skill and Claude `/ap-*` commands (scope aware).

### 1.7 Quality & release readiness
- [ ] Unit tests for resolver/lock/overlay.
- [ ] Golden tests for adapter plan output snapshots.
- [ ] Ensure CI green on macOS/Linux/Windows.

## 2. Documentation
- [ ] Keep `docs/SPEC.md` as the source of truth; update only when behavior changes.
- [ ] Update `README.md` usage examples when the CLI is functional.
