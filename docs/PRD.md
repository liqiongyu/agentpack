# PRD.md

> Current as of **v0.8.0** (2026-01-20). Historical content is tracked in git history.

## 1. Background

You likely use multiple AI coding tools (e.g., Codex CLI, Claude Code). Each of them supports a set of pluggable assets: AGENTS.md, skills, slash commands, prompts, etc.

Common real-world pain points:
- Discovery cost: there are many variants of similar capabilities; you need to filter and compare.
- Maintenance cost: once installed, you keep tweaking; when upstream updates, you need to merge.
- Collaboration cost: multiple machines/projects/tools lead to inconsistent versions and paths.
- Control: you want auditable, rollbackable, reproducible state—not “I’m not sure which version is active right now”.

Agentpack’s positioning: a local “asset control plane” that uses a declarative manifest + overlays + lockfile to unify management, distribution, and rollback of these assets—friendly to both humans and agents (AI-first).

## 2. Product goals

P0 (must-have)
- **Reproducible**: git sources are locked to commit + sha256 via lockfile.
- **Rollbackable**: deploy/rollback snapshots.
- **Safe writes**: only delete managed files; overwriting unmanaged files requires explicit adopt.
- **Composable**: stable `--json` output; mutating commands require `--yes` in `--json` mode.

P1 (strongly recommended)
- Smooth overlay edit/merge experience (sparse overlays + rebase).
- Convert drift into reviewable changes (evolve propose).

## 3. Shipped surface area (current implementation)

Closed-loop capabilities:
- Config repo (manifest + overlays) as the single source of truth
- Lockfile (`agentpack.lock.json`) + store/cache (git checkout)
- plan/diff/deploy: planning, diffs, backed-up writes, and snapshots
- Per-root `.agentpack.manifest.<target>.json`: safe deletes and reliable drift/status
- Overwrite protection: `adopt_update` requires `--adopt`

UX and AI-first:
- Composite commands: `update` (lock+fetch) / `preview` (plan + optional diff)
- Three overlay scopes: global/machine/project
- Sparse overlays + `overlay rebase` (3-way merge)
- `doctor --fix`: reduce accidental commits of manifests
- bootstrap: install operator assets (Codex operator skill + Claude `/ap-*` commands)
- evolve: `propose` (turn drift into overlay proposal branches) + `restore` (create-only restore missing files)
- Governance policy tooling: `policy lint` / `policy lock` / `policy audit` (org distribution + supply-chain guardrails)
- MCP server integration: `agentpack mcp serve` exposes structured read-only tools plus approval-gated mutations

## 4. Non-goals (short term)

- A full discovery system / marketplace (start with git/local sources)
- Complex patch-based overlays (stronger merge/patch model)
- GUI (keep CLI-first; add a lightweight TUI only if needed)

## 5. References

- AGENTS.md: https://agents.md/
- Codex:
  - https://developers.openai.com/codex/guides/agents-md/
  - https://developers.openai.com/codex/skills/
  - https://developers.openai.com/codex/custom-prompts/
- Claude Code:
  - https://code.claude.com/docs/en/slash-commands
