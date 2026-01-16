## ADDED Requirements

### Requirement: Provide a policy audit command
The system SHALL provide a read-only governance audit command:

`agentpack policy audit`

The command SHALL be CI-friendly and SHALL NOT require network access.

In `--json` mode, on success:
- stdout MUST be valid JSON with `ok=true`
- `data` MUST include an audit report derived from `repo/agentpack.lock.json`, including module ids, sources, pinned versions, and content hashes.
- `data` SHOULD include a best-effort change summary derived from git history when available.

#### Scenario: policy audit emits a JSON report
- **GIVEN** `repo/agentpack.lock.json` exists
- **WHEN** the user runs `agentpack policy audit --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.modules[]` contains at least one module entry
