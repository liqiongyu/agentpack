# Change: Add Cursor target

## Why
Cursor supports project “Rules” stored under `.cursor/rules/*.mdc`. Adding a first-party `cursor` target lets agentpack manage these rule files from existing `instructions` modules, keeping the AI-first loop consistent across tools.

## What Changes
- Add built-in target `cursor` (files mode).
- Render `instructions` modules to Cursor rule files under `<project_root>/.cursor/rules`.
- Extend conformance tests to include Cursor.
- Update docs (`docs/SPEC.md`, `docs/CLI.md`, `docs/CONFIG.md`, `docs/TARGETS.md`) to document mapping and options.

## Impact
- Specs: `openspec/specs/agentpack/spec.md` (targets + conformance), `docs/SPEC.md` target adapter details.
- Code: `src/config.rs`, `src/cli/util.rs`, `src/target_adapters.rs`, `src/engine.rs`.
- Tests: `tests/conformance_targets.rs`.
