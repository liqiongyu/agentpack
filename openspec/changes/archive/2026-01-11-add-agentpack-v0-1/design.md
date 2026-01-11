# Design Notes (v0.1)

This change implements the v0.1 system described by `docs/ARCHITECTURE.md` and `docs/SPEC.md`.

## Naming: config repo vs project root
- `config_repo`: contains `agentpack.yaml`, `overlays/`, `projects/<project_id>/...`.
- `project_root`: the target project whose `.claude/` and repo-scoped `.codex/` paths may be written.

The CLI should make this explicit in `--json` outputs.

## Ownership & deletion
- Agentpack should never delete arbitrary files from target directories.
- Deletions are limited to paths that were previously written by agentpack and recorded in the latest deployment snapshot.

## Overlay drift warnings (baseline hashes)
- When creating/editing an overlay, record upstream baseline file hashes for the selected files.
- On subsequent lockfile/source resolution changes, if upstream hash changes for a covered file, emit a warning requiring manual review.

## Determinism requirements
- Lockfile must be stable: deterministic file enumeration, path normalization, stable ordering, sha256 computed from raw bytes.
- `--json` output is treated as a public API contract: additive-only changes.
