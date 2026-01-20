# agentpack Delta

## ADDED Requirements

### Requirement: doctor logic is centralized

The repository SHALL centralize `doctor` report generation (root checks, gitignore checks/fix planning, and overlay layout warnings) in a handlers module so callers can reuse a single implementation, without changing user-facing behavior.

#### Scenario: doctor handler refactor preserves behavior
- **WHEN** doctor logic is reorganized
- **THEN** existing CLI tests continue to pass
