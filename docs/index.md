# Agentpack Docs

> Language: English | [Chinese (Simplified)](zh-CN/index.md)

This page is the single “start here” entrypoint for Agentpack documentation.

## Choose your path

### 1) From scratch (first-time setup)

- Follow the Quickstart to install, initialize a config repo, and do your first deploy:
  - `QUICKSTART.md`
- Learn the daily loop (update → preview → deploy → status → rollback):
  - `WORKFLOWS.md`

### 2) Adopt existing assets (import)

- If you already have skills/prompts/commands on disk and want Agentpack to manage them:
  - CLI reference: `CLI.md#import`
  - Daily workflow context: `WORKFLOWS.md`

## Common workflows

- Local customization with overlays (including patch overlays):
  - `OVERLAYS.md` (see `overlay edit --kind patch`)
- Drift → proposal → review → merge:
  - `WORKFLOWS.md`
  - `EVOLVE.md`
- AI-first bootstrap (operator assets for Codex / Claude Code):
  - `BOOTSTRAP.md`

## Automation / integrations

- Stable `--json` envelope contract and examples:
  - `JSON_API.md`
  - `ERROR_CODES.md`
- Codex MCP integration (`agentpack mcp serve`):
  - `MCP.md`

## Reference

- CLI reference: `CLI.md`
- Config reference (`agentpack.yaml`): `CONFIG.md`
- Targets reference: `TARGETS.md`
