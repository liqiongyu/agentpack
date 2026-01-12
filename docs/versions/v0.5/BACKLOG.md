# BACKLOG.md

> Current as of **v0.5** (2026-01-12). Historical snapshots live under `docs/versions/`.

## Status
- v0.5 milestone: **done** (see `CHANGELOG.md` for the concrete shipped surface area).

## Next (v0.6+)
### Targets & ecosystem
- Add more targets (e.g. Cursor / VS Code), using the TargetAdapter + conformance harness as the contribution gate.
- Expand target docs and examples as new adapters land.

### UX & ergonomics
- Improve operator workflows (more guided “doctor → update → preview → deploy” flows; richer, actionable warnings).
- Consider a lightweight TUI for browsing plan/diff/status and recent snapshots.

### Evolve & overlays
- Stronger overlay authoring (patch-based overlays / 3-way merge) with clear conflict reporting.
- More automated “evolve” helpers, while keeping writes explicit and reviewable (PR-first).

## Notes
- `docs/` is the latest; versioned snapshots live in `docs/versions/vX.Y/`.
