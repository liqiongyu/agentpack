# Lightweight TUI (Design)

This document captures the design decisions for adding an optional, lightweight TUI to Agentpack.

## Goals

- Provide an immersive but **non-intrusive** terminal UI for quickly browsing:
  - `plan`
  - `diff`
  - `status`
- Allow a user to trigger **safe apply** from the UI (still requiring explicit confirmation).
- Reuse existing engine/CLI logic (avoid duplicating business rules).

## Non-goals

- Replacing the CLI as the primary automation interface.
- Introducing a new contract: TUI MUST follow existing CLI semantics (including confirmations and stable error codes).

## Command shape

- Subcommand: `agentpack tui`
- The TUI is feature-gated; build with `--features tui` to enable the command.

## Feature gate & dependency strategy

- Cargo feature: `tui`
  - **Default: off**, to avoid dependency bloat for users who only need the CLI.
  - TUI-only dependencies MUST be optional and gated by this feature.
- Suggested dependencies (behind `tui`):
  - `ratatui` (UI framework)
  - `crossterm` (terminal backend)

## Safety model (apply/rollback)

- Any mutating action initiated from the TUI MUST require explicit confirmation, equivalent to the CLI `--yes` semantics.
- Failures MUST surface stable error codes and actionable next steps (same as CLI JSON envelope).

## Testing & CI/release

- CI MUST keep validating the TUI build:
  - At minimum: compile-check via existing “all features” linting.
  - Once TUI tests exist: add a test job that runs with `--all-features` (or at least `--features tui`).
- Release artifacts MAY ship with `tui` enabled, but the crate feature default remains off.

## Usage

The TUI requires an interactive terminal (TTY).

### Key bindings

- Quit: `q` or `Esc`
- Switch tabs: `1`/`2`/`3` (or `p`/`d`/`s`, or `←`/`→`)
- Scroll: `↑`/`↓` (or `j`/`k`), `PgUp`/`PgDn`, `Home`/`End`
