# agentpack Delta

## ADDED Requirements

### Requirement: status drift logic is centralized

The repository SHALL centralize `status` drift computation (manifest reading, drift detection, and related warnings) in a handlers module so callers can reuse a single implementation, without changing user-facing behavior.

#### Scenario: status handler refactor preserves behavior
- **WHEN** status drift logic is reorganized
- **THEN** existing CLI and TUI tests continue to pass
