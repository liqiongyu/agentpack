# Agentpack

> Language: English | [Chinese (Simplified)](README.zh-CN.md)

Agentpack is an AI-first local “asset control plane” for managing and deploying:
- `AGENTS.md` / instructions
- Agent Skills (`SKILL.md`)
- Claude Code slash commands (`.claude/commands`)
- Codex custom prompts (`~/.codex/prompts`)

## Documentation

- Docs index: `docs/README.md` (English), `docs/zh-CN/README.md` (Chinese)
- Start here: `docs/QUICKSTART.md`
- Daily workflow: `docs/WORKFLOWS.md`
- CLI reference: `docs/CLI.md`
- Codex MCP wiring: `docs/MCP.md`

## Installation

### Cargo

```bash
cargo install agentpack --locked
```

If crates.io install is not available yet, install from source:

```bash
cargo install --git https://github.com/liqiongyu/agentpack --tag v0.5.0 --locked
```

### Prebuilt binaries

GitHub Releases: https://github.com/liqiongyu/agentpack/releases

## Quickstart (v0.5)

```bash
agentpack init
agentpack update
agentpack preview --diff
agentpack deploy --apply --yes
```

For a fuller walkthrough, see `docs/README.md`. For automation, see `docs/JSON_API.md`.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## Contributing

Start with `AGENTS.md` and `CONTRIBUTING.md`.
