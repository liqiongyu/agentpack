# README.md (v0.2)

Agentpack is an AI-first local “asset control plane” for managing and deploying agent assets across tools and projects, such as:

- Project instructions (`AGENTS.md`)
- Agent skills (`SKILL.md` + optional scripts)
- Claude Code slash commands (`.claude/commands`)
- Codex custom prompts (`~/.codex/prompts`)

## Why Agentpack

AI coding users typically maintain many prompts/commands/skills that evolve over time.
Agentpack makes them:

- **Versioned**: stored in a local git repo
- **Composable**: global + machine + project overlays
- **Deployable**: to multiple tools with target adapters
- **Safe**: deploy only manages files it owns (manifest), supports rollback
- **AI-first**: stable `--json` outputs and bootstrap assets for tool integration

## Quickstart

```bash
# 1) init repo
agentpack init

# 2) add modules
agentpack add instructions local:modules/instructions/base --id instructions:base --tags base
agentpack add command local:modules/claude-commands/ap-plan.md --id command:ap-plan --tags operator --targets claude_code

# 3) lock & fetch (if you use git sources)
agentpack lock
agentpack fetch

# 4) plan / diff / deploy
agentpack plan --profile default
agentpack diff --profile default
agentpack deploy --profile default --apply

# 5) drift & rollback
agentpack status --profile default
agentpack rollback --to <snapshot_id> --apply
