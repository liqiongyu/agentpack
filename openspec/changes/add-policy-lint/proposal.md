## Why

Organizations want a CI-friendly way to enforce “AI coding asset hygiene” without changing personal-user defaults.

The governance layer is opt-in (`agentpack policy ...`) and should provide a first, safe step: a read-only lint command that reports violations in a machine-readable way.

## What changes

- Add a new CLI namespace: `agentpack policy ...`
- Add `agentpack policy lint` (read-only) with `--json` output suitable for CI gating.
- Introduce a stable error code for lint failures (policy violations).

## Scope (initial checks)

`policy lint` will initially check:
- **Skill frontmatter completeness**: every `SKILL.md` must include YAML frontmatter with required fields (at least `name` and `description`).
- **Claude command allowed-tools**: command markdown that uses the bash tool must declare `allowed-tools` that includes `Bash(...)`.
- **Dangerous defaults**: disallow “mutating agentpack commands” being used without explicit `--json --yes` semantics in automation-oriented command files.

## Non-goals

- No changes to core commands (`plan/diff/deploy/...`) or their behavior.
- No mandatory governance config file for existing users.
- No network access requirements (lint should be best-effort and CI-friendly; missing optional dependencies may be reported as issues).

## Impact

- New opt-in command namespace (`agentpack policy ...`).
- New stable error code for policy violations in `--json` mode.
- New tests and documentation for the governance lint contract.
