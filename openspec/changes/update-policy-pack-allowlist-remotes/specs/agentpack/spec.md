## MODIFIED Requirements

### Requirement: Governance supply_chain_policy can allowlist git remotes

The governance config (`repo/agentpack.org.yaml`) SHALL support an optional supply chain policy section:

- `supply_chain_policy.allowed_git_remotes: string[]`

When `allowed_git_remotes` is configured and non-empty, `agentpack policy lint` MUST validate that:
- every git-sourced module in `repo/agentpack.yaml` uses a git remote that matches at least one allowlist entry, and
- if `policy_pack.source` is a git source, its remote matches at least one allowlist entry.

The allowlist match MUST be case-insensitive and SHOULD treat common git URL forms as equivalent (e.g. `https://github.com/org/repo.git` and `git@github.com:org/repo.git`).

Violations MUST be reported as standard `policy lint` issues and fail with `E_POLICY_VIOLATIONS`.

#### Scenario: policy lint fails when a policy pack uses a non-allowlisted remote
- **GIVEN** `repo/agentpack.org.yaml` configures `supply_chain_policy.allowed_git_remotes`
- **AND** `repo/agentpack.org.yaml` configures a git `policy_pack.source` whose remote does not match the allowlist
- **WHEN** the user runs `agentpack policy lint --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_POLICY_VIOLATIONS`
- **AND** `errors[0].details.issues[]` includes an issue for the non-allowlisted policy pack remote
