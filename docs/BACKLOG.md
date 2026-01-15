# BACKLOG.md

> Current as of **v0.6.0** (2026-01-15). Historical content is tracked in git history.

## Status

- v0.5 milestone: a round of “daily-usable + AI-first loop” convergence (composite commands, overlay rebase, adopt protection, evolve restore, etc.).
- For concrete shipped changes, see `CHANGELOG.md`.

## Next (candidates for v0.6+)

### Targets & ecosystem
- Add more targets (JetBrains / Zed / etc.), gated by: TargetAdapter + conformance tests.
- For each new target: mapping rules, examples, migration notes.

### UX & ergonomics
- Stronger `status` output (optional summary, grouped by root, actionable suggestions).
- Richer but still script-friendly warnings (with actionable commands where possible).
- Consider a lightweight TUI (browse plan/diff/status/snapshots) while keeping the core usable in non-interactive mode.

### Overlays & evolve
- Patch-based overlays (optional): make small text edits easier to merge and conflicts more readable.
- Expand evolve propose coverage: better attribution for multi-module aggregated outputs (beyond AGENTS.md) and more structured skipped reasons.
- Provide clearer “next command” suggestions in evolve output (good for operator assets).

### Engineering
- CLI golden tests (regression coverage for JSON output/error codes).
- Stronger conformance harness (temp roots, cross-platform path cases).
- Keep docs consolidated (legacy `docs/versions/` removed; rely on git history for iteration tracking).
