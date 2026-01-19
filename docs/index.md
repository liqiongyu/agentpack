# Agentpack Docs

> Language: English | [Chinese (Simplified)](zh-CN/index.md)

This page is the single “start here” entrypoint for Agentpack documentation.

## Choose your path

### 1) From scratch (first-time setup)

- Follow the Quickstart to install, initialize a config repo, and do your first deploy:
  - `tutorials/quickstart.md`
- Learn the daily loop (update → preview → deploy → status → rollback):
  - `howto/workflows.md`

### 2) Adopt existing assets (import)

- If you already have skills/prompts/commands on disk and want Agentpack to manage them:
  - CLI reference: `reference/cli.md#import`
  - Daily workflow context: `howto/workflows.md`

## Common workflows

- Local customization with overlays (including patch overlays):
  - `explanation/overlays.md` (see `overlay edit --kind patch`)
- Drift → proposal → review → merge:
  - `howto/workflows.md`
  - `howto/evolve.md`
- AI-first bootstrap (operator assets for Codex / Claude Code):
  - `howto/bootstrap.md`

## Automation / integrations

- Stable `--json` envelope contract and examples:
  - `reference/json-api.md`
  - `reference/error-codes.md`
- Codex MCP integration (`agentpack mcp serve`):
  - `howto/mcp.md`

## Reference

- CLI reference: `reference/cli.md`
- Config reference (`agentpack.yaml`): `reference/config.md`
- Targets reference: `reference/targets.md`
