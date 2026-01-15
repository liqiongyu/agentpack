## 1. Spec (OpenSpec)
- [x] Define `agentpack tui` (read-only) behavior in the delta spec under `openspec/changes/add-tui-readonly/specs/agentpack-cli/spec.md`.

## 2. Validation
- [x] `openspec validate add-tui-readonly --strict --no-interactive`

## 3. Implementation (read-only TUI)
- [x] Add an optional `tui` Cargo feature and gate TUI-only dependencies behind it.
- [x] Add `agentpack tui` subcommand (only present when built with `tui` feature).
- [x] Implement read-only UI with three views (plan/diff/status) by reusing existing engine/CLI logic.
- [x] Add at least one integration test for the non-UI core wiring/logic (UI details are out of scope).
- [x] Document basic usage/exit keys in `docs/TUI.md` (update as needed).

## 4. Archive (after deploy)
- [ ] Archive the change via `openspec archive add-tui-readonly --yes` in a separate PR after the implementation is merged.
