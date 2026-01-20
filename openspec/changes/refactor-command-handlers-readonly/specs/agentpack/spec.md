# agentpack Delta

## ADDED Requirements

### Requirement: Read-only command planning logic is centralized

The repository SHALL keep the shared read-only planning pipeline (desired-state rendering + managed-path resolution + plan computation) centralized in a handlers module to reduce drift between CLI and TUI implementations, without changing user-facing behavior.

#### Scenario: readonly handler refactor preserves behavior
- **WHEN** read-only command logic is reorganized
- **THEN** the existing CLI, TUI, and journey tests continue to pass
