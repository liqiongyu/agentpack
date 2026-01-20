# Change: Refactor doctor command into a shared handler

## Why
The `doctor` command currently contains substantial business logic inside the CLI layer. Centralizing it in `src/handlers/` reduces duplication and is a step toward shared logic across CLI/MCP/TUI (M3-REF-004), while preserving existing user-facing behavior and contracts.

## What Changes
- Add `src/handlers/doctor.rs` for core doctor checks (target roots, gitignore detection/fix planning, overlay layout warnings).
- Refactor CLI `doctor` to call the handler for report generation.

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/handlers/doctor.rs`, `src/cli/commands/doctor.rs`
- Compatibility: no CLI/JSON behavior changes intended
