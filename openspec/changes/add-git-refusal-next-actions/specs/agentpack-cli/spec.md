## ADDED Requirements

### Requirement: Git refusal errors are machine-actionable
When a `--json` invocation is refused due to a git precondition failure, the system SHALL include additive, machine-actionable fields under `errors[0].details`:
- `reason_code: string` (stable, enum-like)
- `next_actions: string[]` (stable, enum-like action identifiers)

This requirement applies to these stable error codes:
- `E_GIT_REPO_REQUIRED`
- `E_GIT_WORKTREE_DIRTY`
- `E_GIT_DETACHED_HEAD`
- `E_GIT_REMOTE_MISSING`
- `E_GIT_NOT_FOUND`

#### Scenario: sync --json returns machine-actionable git refusal details
- **GIVEN** the config repo is missing required git preconditions for `sync` (e.g. not a git repo)
- **WHEN** the user runs `agentpack sync --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` is one of the git refusal error codes above
- **AND** `errors[0].details.reason_code` is present
- **AND** `errors[0].details.next_actions` is present
