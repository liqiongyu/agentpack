# agentpack-cli (delta)

## ADDED Requirements

### Requirement: skill modules require valid SKILL.md frontmatter
The system SHALL validate `skill` modules’ `SKILL.md` during module materialization. `SKILL.md` SHALL start with YAML frontmatter (`--- ... ---`) and the frontmatter SHALL include:
- `name` as a non-empty string
- `description` as a non-empty string

When invoked with `--json`, invalid skill frontmatter SHALL fail with stable code `E_CONFIG_INVALID` and include details identifying the module and field(s) to fix.

#### Scenario: plan --json fails on invalid SKILL.md frontmatter
- **GIVEN** a `skill` module whose `SKILL.md` is missing YAML frontmatter
- **WHEN** the user runs `agentpack plan --target codex --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIG_INVALID`
