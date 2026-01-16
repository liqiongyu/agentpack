# Agentpack Docs

> Language: English | [Chinese (Simplified)](zh-CN/README.md)

This documentation set serves two audiences:
- **Users (just want it to work)**: install, configure, preview, deploy, rollback, and capture local edits via overlays.
- **Contributors (changing code / adding targets)**: understand the engine, data model, the `--json` contract, and conformance tests.

English docs under `docs/` are canonical. Chinese (Simplified) translations for user docs live under `docs/zh-CN/` and may lag slightly behind.

If you just want to start using Agentpack, begin with **Quickstart**.

## User docs (recommended order)

1) **Quickstart**: `QUICKSTART.md`
- Get from 0 → first successful deploy in ~30 minutes.

2) **Daily workflows**: `WORKFLOWS.md`
- Update (update) → Preview (preview) → Apply (deploy --apply)
- Drift (status) → Proposal (evolve propose) → Review → Merge

3) **CLI reference**: `CLI.md`
- Global flags and per-command usage/examples.

4) **Config & modules**: `CONFIG.md`
- `agentpack.yaml` (profiles/targets/modules) and source specs (local/git).

5) **Targets (Codex / Claude Code)**: `TARGETS.md`
- Where files are written, options, and limitations (especially prompts = user scope only).

6) **Overlays**: `OVERLAYS.md`
- global/machine/project overlays
- `overlay edit --sparse/--materialize` and `overlay rebase` (3-way merge)

7) **Bootstrap (AI operator assets)**: `BOOTSTRAP.md`
- Install operator assets so agents can self-serve with agentpack.

8) **Evolve loop**: `EVOLVE.md`
- record/score/explain/propose/restore

9) **Troubleshooting**: `TROUBLESHOOTING.md`
- Common error codes, conflicts, permissions, Windows path gotchas, etc.

10) **MCP (Codex integration)**: `MCP.md`
- Configure `agentpack mcp serve` as an MCP server in Codex.

## Contracts & references (automation + contributors)

- **SPEC (implementation-aligned source of truth)**: `SPEC.md`
- **JSON output contract**: `JSON_API.md`
- **Stable error code registry**: `ERROR_CODES.md`
- **Architecture overview**: `ARCHITECTURE.md`
- **Target SDK (how to add a target)**: `TARGET_SDK.md`
- **Target conformance**: `TARGET_CONFORMANCE.md`

## Maintainer / project ops

- Release process: `RELEASING.md`
- Dependency & security checks: `SECURITY_CHECKS.md`
- GitHub setup checklist (manual / admin-only): `GITHUB_SETUP.md`
- Governance layer (opt-in): `GOVERNANCE.md` (includes `agentpack policy lint|lock`)
- Product docs (background/roadmap): `PRD.md`, `BACKLOG.md`
- Unified spec + epics + backlog (Codex-ready): `Agentpack_Spec_Epics_Backlog_CodexReady.md`
- Roadmap + spec + epics + backlog (Codex-ready, v0.6): `Agentpack_Roadmap_Spec_Epics_Backlog_CodexReady_v0.6.md`
- Executable workplan (for Codex CLI): `CODEX_WORKPLAN.md`

## External references

- AGENTS.md: https://agents.md/
- Codex:
  - AGENTS.md discovery: https://developers.openai.com/codex/guides/agents-md/
  - Skills: https://developers.openai.com/codex/skills/
  - Custom prompts: https://developers.openai.com/codex/custom-prompts/
- Claude Code:
  - Slash commands: https://code.claude.com/docs/en/slash-commands
