## ADDED Requirements

### Requirement: Governance can require lockfile pinning for git modules
The governance config (`repo/agentpack.org.yaml`) SHALL support an optional boolean:

- `supply_chain_policy.require_lockfile: boolean`

When `require_lockfile=true` and the repo manifest (`repo/agentpack.yaml`) contains at least one enabled git-sourced module, `agentpack policy lint` MUST enforce that:
- `repo/agentpack.lock.json` exists and is valid, and
- the lockfile contains entries for the enabled git modules, and
- each locked git module’s `resolved_source.git.url` matches the configured module git URL.

Violations MUST be reported as policy lint issues and fail with `E_POLICY_VIOLATIONS`.

#### Scenario: policy lint fails when lockfile is missing for git modules
- **GIVEN** `repo/agentpack.org.yaml` configures `supply_chain_policy.require_lockfile=true`
- **AND** `repo/agentpack.yaml` contains at least one enabled git-sourced module
- **AND** `repo/agentpack.lock.json` is missing
- **WHEN** the user runs `agentpack policy lint --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_POLICY_VIOLATIONS`
