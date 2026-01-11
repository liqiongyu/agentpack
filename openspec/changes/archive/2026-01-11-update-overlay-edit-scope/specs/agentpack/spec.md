# agentpack (delta)

## ADDED Requirements

### Requirement: Overlay editing
The system MUST support creating overlay skeletons across overlay scopes:
- `global`: `repo/overlays/<moduleId>/...`
- `machine`: `repo/overlays/machines/<machineId>/<moduleId>/...`
- `project`: `repo/projects/<projectId>/overlays/<moduleId>/...`

#### Scenario: Machine overlay skeleton is created
- **GIVEN** a module `<moduleId>` exists and resolves to an upstream root
- **WHEN** the user runs `agentpack overlay edit <moduleId> --scope machine`
- **THEN** the directory `repo/overlays/machines/<machineId>/<moduleId>/` exists
- **AND** it contains the upstream content (copied)
- **AND** it contains `.agentpack/baseline.json`
