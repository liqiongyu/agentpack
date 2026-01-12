# agentpack (delta)

## ADDED Requirements

### Requirement: Lockfile local paths are repo-relative
When generating `agentpack.lock.json`, the system SHALL record `resolved_source.local_path.path` as a stable, repo-relative path so the lockfile remains portable across machines.

#### Scenario: Lockfile local_path is stable across machines
- **GIVEN** a module uses a `local_path` source inside the agentpack repo
- **WHEN** the user runs `agentpack lock`
- **THEN** the lockfile stores `resolved_source.local_path.path` without embedding an absolute machine path
