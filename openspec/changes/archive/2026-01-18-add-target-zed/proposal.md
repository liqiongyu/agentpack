# Change: add target zed

## Why
Zed supports project-level AI “rules” via a repo-root `.rules` file (and other compatible filenames). Agentpack can already generate Zed-compatible rules today via existing targets (e.g., `vscode` → `.github/copilot-instructions.md`, `codex` → `AGENTS.md`), but a dedicated `zed` target:
- provides a first-class mapping to Zed’s preferred `.rules` filename, and
- keeps drift/rollback boundaries explicit via a target-specific manifest.

With per-target manifest filenames shipped, `zed` can safely share the repo root as a managed root alongside other targets (e.g., `codex` project instructions).

## What Changes
- Add a new built-in target adapter `zed` (Cargo feature: `target-zed`).
- Map `instructions` modules into a single repo-root rules file: `<project_root>/.rules`.
- Write a per-target manifest at `<project_root>/.agentpack.manifest.zed.json` for safe deletes and drift detection.
- Add target conformance coverage and update docs.

## Impact
- Specs: `agentpack`, `agentpack-cli`
- Code: target registry + config validation + engine rendering + conformance tests
- CLI output: `help --json` `targets[]` includes `zed` when compiled (additive)

## Non-Goals
- Managing Zed’s user-level Rules Library files.
- Managing `.zed/settings.json` / editor configuration wiring.
- Adding `evolve propose` support for `.rules` drift (can be added later).
