## 1. Implementation (v0.2)

### 1.1 Manifest-based deploy safety (P0)
- [ ] Add a target manifest model (`.agentpack.manifest.json`) with stable schema and hashing.
- [ ] Write/update manifests during `deploy --apply` (and `bootstrap --apply` if applicable).
- [ ] Compute deletes from manifests (not from snapshots), and never delete unmanaged files.
- [ ] Update `status` drift detection to read manifests and classify: changed/missing/extra.

### 1.2 Multi-machine sync (P0)
- [ ] Add `agentpack remote set` to configure the config repo git remote.
- [ ] Add `agentpack sync` to run pull/rebase/push (best-practice wrapper, no magic).

### 1.3 Machine overlay + doctor (P0)
- [ ] Add `doctor` command: machineId + target path validation (existence/writable) with suggestions.
- [ ] Add global `--machine <id>` and apply machine overlays between global and project overlays.

### 1.4 AI-first JSON schema upgrades (P0)
- [ ] Add `schemaVersion` to `--json` envelope and keep backwards-compatible fields.
- [ ] Ensure core commands include warnings/errors consistently in `--json` mode.

### 1.5 Evolution loop + explainability (P1)
- [ ] Add `record` (stdin JSON → append-only event log).
- [ ] Add `score` (module health summary: failure rate, recency, rollback count if available).
- [ ] Add `evolve propose` (generate reviewable patch files under state; no auto-apply).
- [ ] Add `explain plan|diff|status` (provenance / “why” output).

### 1.6 Tests + docs
- [ ] Add tests for manifest delete protection and drift classification.
- [ ] Update docs (`docs/*`, `openspec/project.md`, templates) to reflect v0.2 behavior.
