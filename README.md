# Agentpack

> A declarative, safe control plane for deploying coding-agent assets across tools.

> Language: English | [Chinese (Simplified)](README.zh-CN.md)

Agentpack is an AI-first local “asset control plane” for managing and deploying:
- `AGENTS.md` / instructions
- Agent Skills (`SKILL.md`)
- Claude Code slash commands (`.claude/commands`)
- Codex custom prompts (`~/.codex/prompts`)

## Documentation

- Docs entrypoint: `docs/index.md` (English), `docs/zh-CN/index.md` (Chinese)
- Quickstart: `docs/tutorials/quickstart.md`
- 5-minute demo (safe preview): `docs/tutorials/demo-5min.md`
- Daily workflow: `docs/howto/workflows.md`
- CLI reference: `docs/reference/cli.md`
- Codex MCP wiring: `docs/howto/mcp.md`

## Architecture (high level)

```mermaid
flowchart TD
  M[agentpack.yaml<br/>manifest] --> C[Compose & materialize<br/>(per module)]
  L[agentpack.lock.json<br/>lockfile] --> C
  O[overlays<br/>(global / machine / project)] --> C

  C --> R[Render desired state<br/>(per target)]
  R --> P[Plan / Diff]
  P -->|dry run| OUT[Human output / JSON envelope]
  P -->|deploy --apply| A[Apply (writes)]

  A --> MF[Write target manifest<br/>.agentpack.manifest.&lt;target&gt;.json]
  A --> SS[Create snapshot<br/>state/snapshots/]
  A --> EV[Record events<br/>state/logs/]

  SS --> RB[Rollback]
```

For details, see `docs/explanation/architecture.md`.

## Installation

### Cargo

```bash
cargo install agentpack --locked
```

If crates.io install is not available yet, install from source:

```bash
cargo install --git https://github.com/liqiongyu/agentpack --tag v0.8.0 --locked
```

### Prebuilt binaries

GitHub Releases: https://github.com/liqiongyu/agentpack/releases

## Quickstart

```bash
agentpack init
agentpack update
agentpack preview --diff
agentpack deploy --apply --yes
```

For a fuller walkthrough, see `docs/index.md`. For automation, see `docs/reference/json-api.md`.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## Contributing

Start with `AGENTS.md` and `CONTRIBUTING.md`.
