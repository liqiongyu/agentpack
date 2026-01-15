## 1. Spec (OpenSpec)
- [x] Define `agentpack tui` safe apply behavior in the delta spec under `openspec/changes/add-tui-safe-apply/specs/agentpack-cli/spec.md`.

## 2. Validation
- [x] `openspec validate add-tui-safe-apply --strict --no-interactive`

## 3. Implementation (TUI apply)
- [ ] Add an explicit “apply” action to `agentpack tui` that performs the equivalent of `deploy --apply` for the current profile/target.
- [ ] Require explicit confirmation in the UI before writing.
- [ ] On apply failure, display stable error codes/messages (and details when present) in the UI.
- [ ] Update `docs/TUI.md` with the apply keybinding and confirmation UX.
- [ ] Add at least one test covering the “no silent writes” guarantee (headless core logic preferred; UI details out of scope).

## 4. Archive (after deploy)
- [ ] Archive the change via `openspec archive add-tui-safe-apply --yes` in a separate PR after the implementation is merged.
