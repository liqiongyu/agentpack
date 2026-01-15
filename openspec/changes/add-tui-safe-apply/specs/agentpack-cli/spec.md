# agentpack-cli (delta)

## ADDED Requirements

### Requirement: tui supports safe apply with explicit confirmation
When built with the `tui` feature enabled, the system SHALL allow users to trigger an apply from within `agentpack tui`, equivalent in semantics to `agentpack deploy --apply` for the current `--profile` / `--target` selection.

The TUI apply flow MUST:
- require explicit confirmation inside the UI before performing any writes, and
- avoid any silent writes.

If apply fails due to a user-facing error, the TUI SHALL display the stable error code and message, and SHOULD display `details` when present.

#### Scenario: user cancels apply
- **GIVEN** the user is running `agentpack tui`
- **WHEN** the user triggers apply
- **AND** the user declines confirmation
- **THEN** no filesystem writes occur

#### Scenario: apply fails and error code is shown
- **GIVEN** the user is running `agentpack tui`
- **WHEN** the user triggers apply and confirms
- **AND** apply fails with a stable error code (e.g. `E_ADOPT_CONFIRM_REQUIRED`)
- **THEN** the UI displays the error code and message
