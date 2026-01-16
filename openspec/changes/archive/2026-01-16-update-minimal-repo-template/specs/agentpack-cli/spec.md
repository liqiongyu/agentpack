## ADDED Requirements

### Requirement: Repository ships a minimal config repo template

The repository SHALL include a copy/pasteable minimal config repo template under `docs/examples/minimal_repo/` that contains:
- a valid `agentpack.yaml`
- an `instructions` module (`modules/instructions/base/AGENTS.md`)
- a `prompt` module (`modules/prompts/...`)
- a `skill` module with `SKILL.md` (`modules/skills/.../SKILL.md`)

The docs SHOULD provide a one-screen command sequence that uses this example (and recommends installing operator `/ap-*` commands via `agentpack bootstrap`).

#### Scenario: minimal example repo can run plan
- **WHEN** the user runs `agentpack --repo docs/examples/minimal_repo plan`
- **THEN** the command exits zero
