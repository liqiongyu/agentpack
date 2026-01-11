## 1. Implementation (v0.1)

### 1.1 Core model & repo/store/state layout
- [x] Implement `AGENTPACK_HOME` resolution (macOS/Linux/Windows) and directory layout (`repo/`, `store/`, `state/`, `logs/`).
- [x] Implement config repo loader for `agentpack.yaml` (version/profiles/targets/modules validation).
- [x] Implement project context detection (cwd, git root, git remote) and stable `project_id`.

### 1.2 Sources, lockfile, and store
- [x] Parse and normalize sources: `local_path` and `git` (url/ref/subdir/shallow).
- [x] Resolve git refs to a concrete commit and compute deterministic file manifests + sha256.
- [x] Generate `agentpack.lock.json` with stable ordering and reproducible content.
- [x] Implement store fetch/cache and sha256 verification.

### 1.3 Overlays
- [x] Implement overlay resolution and composition:
  - upstream -> `repo/overlays/<module_id>/...` -> `repo/projects/<project_id>/overlays/<module_id>/...`
- [x] Implement overlay drift warnings (record upstream baseline for edited files).
- [x] Implement `agentpack overlay edit <module_id> [--project]` (skeleton + editor).

### 1.4 Deploy pipeline
- [x] Implement plan generation (create/update/delete) for each target with reasons.
- [x] Implement diff (text + JSON summary).
- [x] Implement apply with backups and atomic writes (temp -> rename).
- [x] Implement validate (minimal structural checks for each module type).
- [x] Implement state snapshots (`deployments/<id>.json`) and rollback by snapshot id.
- [x] Implement `status` drift detection (expected vs actual).

### 1.5 Target adapters
- [x] Codex adapter: skills (repo + user), prompts (user), AGENTS (global + repo).
- [x] Claude Code adapter (files mode): commands (repo + user) + frontmatter validation.

### 1.6 AI-first bootstrap
- [x] Embed operator assets templates.
- [x] Implement `bootstrap` to install Codex operator skill and Claude `/ap-*` commands (scope aware).

### 1.7 Quality & release readiness
- [x] Unit tests for resolver/lock/overlay.
- [x] Golden tests for adapter plan output snapshots.
- [x] Ensure CI green on macOS/Linux/Windows.

## 2. Documentation
- [x] Keep `docs/SPEC.md` as the source of truth; update only when behavior changes.
- [x] Update `README.md` usage examples when the CLI is functional.
