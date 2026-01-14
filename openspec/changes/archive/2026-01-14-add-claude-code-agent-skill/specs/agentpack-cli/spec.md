# agentpack-cli (delta)

## ADDED Requirements

### Requirement: Claude Code target can deploy Skills (optional)
The system SHALL support deploying `skill` modules to Claude Code skill directories when enabled via `targets.claude_code.options.write_user_skills` and/or `targets.claude_code.options.write_repo_skills`.

#### Scenario: deploy writes a repo Skill when enabled
- **GIVEN** an enabled `skill` module targeting `claude_code`
- **AND** `targets.claude_code.scope=project` and `targets.claude_code.options.write_repo_skills=true`
- **WHEN** the user runs `agentpack --target claude_code deploy --apply`
- **THEN** the Skill is written under `<project_root>/.claude/skills/<skill_name>/...`
- **AND** `<project_root>/.claude/skills/.agentpack.manifest.json` exists

### Requirement: bootstrap can optionally install the Claude operator Skill
The system SHALL optionally install the Claude Code `agentpack-operator` Skill during `agentpack bootstrap` when enabled via `targets.claude_code.options.write_*_skills`.

#### Scenario: bootstrap writes the operator Skill when enabled
- **GIVEN** `targets.claude_code.scope=project` and `targets.claude_code.options.write_repo_skills=true`
- **WHEN** the user runs `agentpack --target claude_code bootstrap --scope project`
- **THEN** `<project_root>/.claude/skills/agentpack-operator/SKILL.md` exists
- **AND** the file contains an `agentpack_version` marker matching the running `agentpack` version
