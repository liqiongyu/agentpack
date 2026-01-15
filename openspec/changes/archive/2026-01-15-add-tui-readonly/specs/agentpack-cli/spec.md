# agentpack-cli (delta)

## ADDED Requirements

### Requirement: optional read-only TUI for plan/diff/status
When built with the `tui` feature enabled, the system SHALL provide an `agentpack tui` subcommand that offers a read-only terminal UI for browsing `plan`, `diff`, and `status` information.

The read-only TUI SHALL:
- reuse existing engine/CLI logic to compute plan/diff/status (no duplicate business rules), and
- avoid filesystem writes by default (any mutating actions are out of scope for this change).

When built without the `tui` feature enabled, the `tui` subcommand MAY be absent.

#### Scenario: tui starts and shows read-only views
- **GIVEN** an agentpack config repo is initialized and has at least one module configured
- **WHEN** the user runs `agentpack tui`
- **THEN** the UI starts successfully
- **AND** it provides views for `plan`, `diff`, and `status`
- **AND** it does not write to target roots by default
