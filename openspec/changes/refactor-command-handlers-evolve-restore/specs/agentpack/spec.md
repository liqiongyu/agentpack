# agentpack Delta

## ADDED Requirements

### Requirement: evolve.restore logic is centralized

The repository SHALL centralize `evolve.restore` business logic (missing-output detection, confirmation guardrails, and write execution) in a handlers module so callers can reuse a single implementation, without changing user-facing behavior.

#### Scenario: evolve.restore handler refactor preserves behavior
- **WHEN** evolve.restore logic is reorganized
- **THEN** existing CLI and journey tests continue to pass
