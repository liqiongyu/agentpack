# agentpack Delta

## ADDED Requirements

### Requirement: Rollback logic is centralized

The repository SHALL centralize rollback business logic and `--json` guardrails in a handlers module so callers reuse a single implementation, without changing user-facing behavior.

#### Scenario: rollback handler refactor preserves behavior
- **WHEN** rollback logic is reorganized
- **THEN** existing CLI, MCP, and journey tests continue to pass
