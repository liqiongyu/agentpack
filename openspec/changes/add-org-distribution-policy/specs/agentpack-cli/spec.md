## ADDED Requirements

### Requirement: policy lint can validate org distribution policy
When `repo/agentpack.org.yaml` configures a `distribution_policy`, `agentpack policy lint` SHALL validate that the repo manifest (`repo/agentpack.yaml`) satisfies the policy.

Violations MUST cause the command to exit non-zero and return `E_POLICY_VIOLATIONS` in `--json` mode.

The violation report SHALL be machine-readable and SHOULD include a stable `rule` identifier per violation (e.g., `distribution_required_targets`, `distribution_required_modules`).

#### Scenario: lint fails when a required target is missing
- **GIVEN** `repo/agentpack.org.yaml` configures `distribution_policy.required_targets=["codex"]`
- **AND** `repo/agentpack.yaml` does not define the `codex` target
- **WHEN** the user runs `agentpack policy lint --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_POLICY_VIOLATIONS`
- **AND** the output includes machine-readable details about the missing target

#### Scenario: lint fails when a required module is missing or disabled
- **GIVEN** `repo/agentpack.org.yaml` configures `distribution_policy.required_modules=["instructions:base"]`
- **AND** `repo/agentpack.yaml` is missing that module id (or has it disabled)
- **WHEN** the user runs `agentpack policy lint --json`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_POLICY_VIOLATIONS`
- **AND** the output includes machine-readable details about the missing/disabled module
