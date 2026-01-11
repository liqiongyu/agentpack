# agentpack (delta)

## ADDED Requirements

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex` and `claude_code` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths
