# Agentpack

> Language: 简体中文 | [English](README.md)

Agentpack 是一个 AI-first 的本地“资产控制平面（asset control plane）”，用于管理与部署：
- `AGENTS.md` / instructions
- Agent Skills（`SKILL.md`）
- Claude Code slash commands（`.claude/commands`）
- Codex custom prompts（`~/.codex/prompts`）

## 文档

- 文档入口：`docs/README.md`（英文）、`docs/zh-CN/README.md`（中文）
- 推荐从：`docs/QUICKSTART.md` 开始
- 日常工作流：`docs/WORKFLOWS.md`
- CLI 参考：`docs/CLI.md`
- Codex MCP 集成：`docs/MCP.md`

## 安装

### Cargo

```bash
cargo install agentpack --locked
```

如果暂时无法从 crates.io 安装，可以从源码安装：

```bash
cargo install --git https://github.com/liqiongyu/agentpack --tag v0.7.0 --locked
```

### 预编译二进制

GitHub Releases: https://github.com/liqiongyu/agentpack/releases

## 快速开始（v0.7）

```bash
agentpack init
agentpack update
agentpack preview --diff
agentpack deploy --apply --yes
```

更完整的上手与阅读顺序见 `docs/README.md`；自动化用 `--json`（契约见 `docs/JSON_API.md`）。

## 开发

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## 贡献

从 `AGENTS.md` 与 `CONTRIBUTING.md` 开始。
