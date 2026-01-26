# Agentpack Docs

> Language: English | [Chinese (Simplified)](zh-CN/index.md)

This page is the single “start here” entrypoint for Agentpack documentation.

## Choose your path

### 1) From scratch (first-time setup)

- Follow the Quickstart to install, initialize a config repo, and do your first deploy:
  - [tutorials/quickstart.md](tutorials/quickstart.md)
- Learn the daily loop (update → preview → deploy → status → rollback):
  - [howto/workflows.md](howto/workflows.md)

### 2) Adopt existing assets (import)

- If you already have skills/prompts/commands on disk and want Agentpack to manage them:
  - CLI reference: [reference/cli.md#import](reference/cli.md#import)
  - Daily workflow context: [howto/workflows.md](howto/workflows.md)

### 3) 5-minute demo (safe preview)

- Run a safe demo in a temporary HOME/AGENTPACK_HOME (no real writes):
  - [tutorials/demo-5min.md](tutorials/demo-5min.md)

### 4) Compare (boundaries vs dotfiles managers)

- If you're evaluating Agentpack vs Stow/chezmoi/yadm:
  - [explanation/compare-dotfiles-managers.md](explanation/compare-dotfiles-managers.md)

### 5) Architecture (how it works)

- Learn how Agentpack composes overlays, renders targets, and applies safely:
  - [explanation/architecture.md](explanation/architecture.md)

## Common workflows

- Local customization with overlays (including patch overlays):
  - [explanation/overlays.md](explanation/overlays.md) (see `overlay edit --kind patch`)
  - [howto/overlays-create-sparse-materialize-rebase.md](howto/overlays-create-sparse-materialize-rebase.md)
- Drift → proposal → review → merge:
  - [howto/workflows.md](howto/workflows.md)
  - [howto/evolve.md](howto/evolve.md)
- AI-first bootstrap (operator assets for Codex / Claude Code):
  - [howto/bootstrap.md](howto/bootstrap.md)

## Automation / integrations

- Stable `--json` envelope contract and examples:
  - [reference/json-api.md](reference/json-api.md)
  - [reference/error-codes.md](reference/error-codes.md)
- Codex MCP integration (`agentpack mcp serve`):
  - [howto/mcp.md](howto/mcp.md)

## Reference

- CLI reference: [reference/cli.md](reference/cli.md)
- Config reference (`agentpack.yaml`): [reference/config.md](reference/config.md)
- Targets reference: [reference/targets.md](reference/targets.md)
