## ADDED Requirements

### Requirement: import mapping rules are deterministic and collision-free

When scanning existing assets, `agentpack import` SHALL map each candidate to a module deterministically, including:
- `module_id` (type-scoped id)
- `module_type`
- `scope` (`user` or `project`)
- `targets`
- `tags`
- destination paths inside the config repo

The command SHOULD include tool tags in `tags` (e.g. `codex`, `claude_code`, `cursor`, `vscode`) based on the candidate source.

The command SHALL ensure that `module_id` values produced in a single import plan are unique. If multiple candidates would otherwise map to the same `module_id`, at least one MUST be reported as skipped (e.g., with `op="skip_invalid"` and a `skip_reason`).

#### Scenario: user-scope prompt mapping is deterministic
- **GIVEN** a temporary home root containing `.codex/prompts/prompt1.md`
- **WHEN** the user runs `agentpack import --home-root <tmp> --json`
- **THEN** `data.plan` contains an item with `module_id="prompt:prompt1"`
- **AND** that item includes `tags` containing `imported`, `user`, and `codex`
- **AND** that item includes `targets` containing `codex`

#### Scenario: user/project collisions do not produce duplicate module ids
- **GIVEN** a project containing a Codex skill under `.codex/skills/foo/SKILL.md`
- **AND** the provided home root also contains `.codex/skills/foo/SKILL.md`
- **WHEN** the user runs `agentpack import --home-root <tmp> --json`
- **THEN** `data.plan` does not contain two items with the same `module_id`
