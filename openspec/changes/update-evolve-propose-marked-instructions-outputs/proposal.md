## Why

`agentpack evolve propose` can map drift in combined `AGENTS.md` back to individual `instructions:*` modules when the file contains per-module section markers.

The VS Code target also produces a combined instructions output at `.github/copilot-instructions.md` and already uses the same marker format when multiple instructions modules are present. However, the current `evolve propose` implementation only attempts marker-based attribution for files named `AGENTS.md`, causing VS Code combined instructions drift to be reported as `multi_module_output` and preventing the “drift → evolve propose → overlay” loop.

## What changes

- Extend marker-based drift attribution in `evolve propose` to work for any combined instructions output that contains valid per-module section markers in both desired and deployed content (not only `AGENTS.md`).
- Add integration tests covering VS Code `.github/copilot-instructions.md`.
- Update documentation to describe combined instructions outputs for both Codex and VS Code.

## Impact

- Behavior: VS Code combined instructions drift becomes proposeable when markers are present; previously skipped items become `candidates`.
- Contract: JSON envelope shape is unchanged; this is effectively additive (more candidates, fewer skipped).
- Risk: low; if markers are missing/unparseable, behavior remains conservative and items continue to be skipped as `multi_module_output`.
