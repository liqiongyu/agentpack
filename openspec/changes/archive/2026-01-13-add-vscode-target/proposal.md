# Change: Add VS Code target

## Why
VS Code / GitHub Copilot supports repo-scoped “custom instructions” and “prompt files” under `.github/`. Adding a first-party `vscode` target lets agentpack generate these assets from `instructions` and `prompt` modules, keeping configuration portable and reproducible.

## What Changes
- Add built-in target `vscode` (files mode).
- Render `instructions` modules into `.github/copilot-instructions.md` (single file; per-module markers when multiple).
- Render `prompt` modules into `.github/prompts/*.prompt.md` (normalize filename to end with `.prompt.md`).
- Extend conformance tests and docs to cover the new target.

## Impact
- Specs: `openspec/specs/agentpack/spec.md` (targets + conformance), `docs/SPEC.md` target adapter details.
- Code: `src/config.rs`, `src/cli/util.rs`, `src/target_adapters.rs`, `src/engine.rs`.
- Tests: `tests/conformance_targets.rs`.
