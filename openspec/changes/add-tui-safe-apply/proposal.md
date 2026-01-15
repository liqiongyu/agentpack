# Change: TUI safe apply

## Why
The read-only TUI is useful for browsing `plan` / `diff` / `status`, but users still need to jump back to the CLI to run `deploy --apply`.

Adding a safe, explicitly-confirmed apply flow inside the TUI improves the “browse → decide → apply” loop without weakening Agentpack’s safety model.

## What Changes
- Extend `agentpack tui` to support triggering an apply (equivalent semantics to `deploy --apply`) from within the UI.
- Require explicit confirmation in the UI before any writes occur.
- On failure, surface stable error codes and details in the UI for actionable debugging.

## Impact
- Affected CLI contract: `agentpack tui` (new mutating action via UI).
- Affected code: TUI event loop + deploy/apply wiring.
- Backward compatibility: default behavior remains read-only unless the user explicitly confirms an apply action.
