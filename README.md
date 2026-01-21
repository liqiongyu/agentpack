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
- Daily workflow: `docs/howto/workflows.md`
- CLI reference: `docs/reference/cli.md`
- Codex MCP wiring: `docs/howto/mcp.md`

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
