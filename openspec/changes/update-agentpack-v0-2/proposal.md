# Change: Update agentpack to v0.2 (manifest + sync + doctor + evolve)

## Why
v0.1 proves the core deploy workflow (`plan -> diff -> deploy -> status -> rollback`) but it still relies on local snapshots for safety, and multi-machine usage is ad-hoc (manual git). v0.2 makes agentpack “production-grade” for daily use: manifest-based safety, multi-machine sync, machine overlays, and an AI-first self-check/observability loop.

## What Changes
- **Deploy safety via manifest**: write `.agentpack.manifest.json` into managed target roots; deletion is restricted to manifest-owned files; `status` reports changed/missing/extra based on manifest.
- **Multi-machine sync**: add `remote` + `sync` commands to standardize `pull/rebase/push` workflows for the config repo.
- **Machine overlays + doctor**: add `doctor` (machineId + path checks) and `--machine` selection; apply overlays in order: upstream → global → machine → project.
- **AI-first interface upgrades**: add `schema_version` to `--json` envelopes; keep warnings/errors consistent.
- **Evolution loop (v0.2 minimal)**: add `record` (event log), `score` (health metrics), and `evolve propose` (generate a reviewable proposal branch, no auto-apply).
- **Explainability**: add `explain` for plan/diff/status provenance.

## Scope
- **In scope (P0/P1)**: manifest, remote/sync, doctor, machine overlays, JSON schema versioning, record/score/evolve propose, explain.
- **Out of scope (P2)**: new target adapters (Cursor/VSCode), TUI, full MCP registry management.

## Risks & Mitigations
- **Risk**: manifest location/format mistakes break safety guarantees.
  - **Mitigation**: treat manifests as first-class outputs; add tests for delete protection and drift classification.
- **Risk**: sync wraps git incorrectly and confuses users.
  - **Mitigation**: keep sync simple (recommended commands + clear errors); never auto-resolve conflicts.
- **Risk**: provenance is noisy or wrong.
  - **Mitigation**: provenance is best-effort and must not affect deploy correctness; surface it via `explain` and `--json` only.

## Acceptance (high-level)
- `deploy --apply` writes manifests and never deletes unmanaged files.
- `status` detects managed-file drift (changed/missing) and reports extra files without error.
- `doctor` outputs machineId and actionable path checks.
- `remote set` + `sync` cover the recommended multi-machine workflow.
- `record/score/evolve propose` work end-to-end without changing deploy behavior.
