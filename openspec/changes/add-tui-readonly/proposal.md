# Change: Read-only TUI (`agentpack tui`)

## Why
Some workflows benefit from a more immersive, faster “browse changes” experience than scrolling CLI output, especially when a plan includes many changes and diffs.

An optional TUI can improve day-to-day usability without replacing the CLI as the primary automation interface.

## What Changes
- Add an optional `agentpack tui` subcommand (feature-gated) for browsing `plan` / `diff` / `status` in a terminal UI.
- Ensure the read-only TUI does not perform writes by default, and reuses existing engine/CLI logic.

## Impact
- Affected CLI contract: new `tui` command (optional; build-feature gated).
- Affected code: new TUI module + CLI command wiring.
- Backward compatibility: unchanged for builds without the `tui` feature; no changes to existing commands.
