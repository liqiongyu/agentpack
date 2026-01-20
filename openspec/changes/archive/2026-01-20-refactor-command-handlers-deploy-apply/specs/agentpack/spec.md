# agentpack Delta

## ADDED Requirements

### Requirement: Deploy apply logic is centralized

The repository SHALL centralize deploy apply business logic (adopt guardrails, manifest-write behavior, and apply execution) in a handlers module so CLI and TUI callers reuse a single implementation, without changing user-facing behavior.

#### Scenario: deploy apply handler refactor preserves behavior
- **WHEN** deploy apply logic is reorganized
- **THEN** existing CLI, TUI, and journey tests continue to pass
