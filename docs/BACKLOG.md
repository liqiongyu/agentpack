# BACKLOG.md

> Current as of **v0.8.0** (2026-01-20). Historical content is tracked in git history.

## Status

- v0.5 milestone: a round of “daily-usable + AI-first loop” convergence (composite commands, overlay rebase, adopt protection, evolve restore, etc.).
- v0.6 milestone: governance policy tooling, MCP server integration (`agentpack mcp serve`), and optional TUI.
- v0.7 milestone: target platformization hardening (per-target manifests) + new built-in targets (`jetbrains`, `zed`).
- v0.8 milestone: MCP tooling refactors (in-process tools) + governance hardening for policy packs.
- For concrete shipped changes, see `CHANGELOG.md`.

## Next (candidates for v0.9+)

### Targets & ecosystem
- Add more targets behind strict feature gates + conformance tests, with clear mapping docs (`docs/TARGET_MAPPING_TEMPLATE.md`).
- Keep “asset rendering” separate from editor configuration wiring (e.g. Zed `.zed/settings.json` integration should be opt-in and not bundled into the core target unless justified).

### UX & ergonomics
- Continue tightening the action-oriented loop: stable `--json` envelopes + stable reason codes + actionable `next_actions`.
- Keep human output readable while making machine output easy to orchestrate.

### Overlays & evolve
- Expand evolve coverage for additional aggregated outputs (beyond repo-root outputs like `AGENTS.md` / `.rules`) while keeping attribution explainable.

### Engineering
- Keep `--json` golden tests and error-code coverage expanding as the surface grows.
- Keep conformance harness coverage expanding (cross-platform paths, permissions, and temp roots).
- Keep docs consolidated (rely on git history for iteration tracking).
