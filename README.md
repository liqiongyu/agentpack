# Agentpack

Agentpack is an AI-first local “asset control plane” for managing and deploying:
- `AGENTS.md` / instructions
- Agent Skills (`SKILL.md`)
- Claude Code slash commands (`.claude/commands`)
- Codex custom prompts (`~/.codex/prompts`)

See product and technical design docs in `docs/`:
- `docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/SPEC.md`, `docs/BACKLOG.md`

## Usage (v0.4)

```bash
# Create (or open) your config repo at $AGENTPACK_HOME/repo
agentpack init

# Optional: configure and sync your config repo across machines
agentpack remote set https://github.com/you/agentpack-config.git
agentpack sync --rebase

# Self-check (machineId + target paths)
agentpack doctor

# Add modules to agentpack.yaml
agentpack add instructions local:modules/instructions/base --id instructions:base --tags base
agentpack add command local:modules/claude-commands/ap-plan.md --id command:ap-plan --tags base --targets claude_code

# Lock and fetch (composite)
agentpack update

# Preview changes (composite: plan + optional diff)
agentpack preview --profile default --diff
agentpack deploy --profile default --apply --yes --json

# Drift + rollback
agentpack status --profile default
agentpack rollback --to <snapshot_id>

# AI-first operator assets
agentpack bootstrap --target all --scope both

# Observability + proposals (v0.2+ loop)
agentpack record < event.json
agentpack score
agentpack explain plan
agentpack evolve propose --scope global --yes
```

For a fuller walkthrough, see `docs/README.md`.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## Contributing

Start with `AGENTS.md` and `CONTRIBUTING.md`.
