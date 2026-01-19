# agentpack-cli Delta

## ADDED Requirements

### Requirement: Journey J6 is covered by an integration test

The project SHALL include a deterministic, offline integration test for Journey J6 (multi-machine sync) that validates `agentpack sync --rebase` against a shared bare git remote.

#### Scenario: two machines can sync a config repo via rebase
- **GIVEN** two independent Agentpack homes share a bare git remote for the config repo
- **WHEN** both machines create commits and run `agentpack sync --rebase --json --yes`
- **THEN** both machines converge on the same git history and file content without network access
