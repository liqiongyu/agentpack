# Agentpack

Agentpack is an AI-first local “asset control plane” for managing and deploying:
- `AGENTS.md` / instructions
- Agent Skills (`SKILL.md`)
- Claude Code slash commands (`.claude/commands`)
- Codex custom prompts (`~/.codex/prompts`)

It gives you a single source of truth (a git repo) + overlays (customization layers) + a lockfile (reproducible versions), then compiles those assets into each tool’s expected filesystem layout (copy/render; no symlinks).

This repo contains both the implementation and the design docs:
- `docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/SPEC.md`, `docs/BACKLOG.md`

## Status

- v0.2: Core loop shipped (lock/fetch, overlay layers, plan/diff/deploy + rollback, per-root manifests, remote sync, bootstrap, record/score/explain/evolve-propose).
- v0.3 (planned): reduce daily friction (composite commands), smoother overlay editing across scopes, better cache-miss behavior, safer AI-first guardrails.

## Usage (v0.2)

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

# Lock and fetch git sources
agentpack lock
agentpack fetch

# Preview / diff / apply
agentpack plan --profile default
agentpack diff --profile default
agentpack deploy --profile default --apply --yes --json

# Drift + rollback
agentpack status --profile default
agentpack rollback --to <snapshot_id>

# AI-first operator assets
agentpack bootstrap --target all --scope both

# Observability + proposals (v0.2 minimal loop)
agentpack record < event.json
agentpack score
agentpack explain plan
agentpack evolve propose --scope global --yes
```

## Upcoming (v0.3 planned)

```bash
# One command for lock+fetch
agentpack update --yes --json

# One command for plan(+diff)
agentpack preview --diff --json

# Overlay editing by scope
agentpack overlay edit skill:git-review --scope machine
```

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## Contributing

Start with `AGENTS.md` and `CONTRIBUTING.md`.
