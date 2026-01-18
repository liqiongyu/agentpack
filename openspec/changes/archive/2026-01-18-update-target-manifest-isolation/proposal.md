# Change: Isolate target manifest filenames per target

## Why
Agentpack currently writes one manifest per managed root at:

`<root>/.agentpack.manifest.json`

This becomes unsafe when multiple targets share the same managed root directory (e.g. repo-root outputs like `AGENTS.md` plus a future `zed` repo-root rules file). The targets would overwrite each other’s manifest, breaking:
- safe-delete boundaries,
- drift detection, and
- rollback restoration for manifests.

## What Changes
- Write a target-specific manifest filename per root:
  - new: `<root>/.agentpack.manifest.<target>.json`
- Preserve backwards compatibility by reading the legacy manifest:
  - legacy: `<root>/.agentpack.manifest.json`
  - only when it belongs to the expected target (based on the manifest `tool` field).
- Update `doctor --fix` and `init --git` to ignore `/.agentpack.manifest*.json` to cover both legacy and new names.
- Update rollback logic to restore all manifest files (legacy and new).

## Non-goals
- Implementing `targets.zed` itself (tracked in #391).
- Changing the JSON envelope schema version (CLI `--json` remains additive only).

## Impact
- Affected OpenSpec capabilities:
  - `openspec/specs/agentpack/spec.md`
  - `openspec/specs/agentpack-cli/spec.md`
- Affected docs:
  - `docs/SPEC.md`, `docs/ARCHITECTURE.md`, `docs/TARGET_CONFORMANCE.md`, and related references
- Tracking issue: #405 (blocks #391)
