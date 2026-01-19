# agentpack Delta

## ADDED Requirements

### Requirement: Overlay implementation is modularized

The repository SHALL keep the overlay implementation split into focused submodules (`layout`, `dir`, `patch`, `rebase`) to reduce coupling and improve maintainability, without changing user-facing behavior.

#### Scenario: overlay refactor preserves behavior
- **WHEN** the overlay implementation is reorganized
- **THEN** the existing overlay/patch/rebase integration tests continue to pass
