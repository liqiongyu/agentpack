# Agentpack

> 面向 AI 编程代理的本地资产控制面：声明式管理与安全部署 AGENTS/Skills/Commands/Prompts。

> Language: 简体中文 | [English](README.md)

Agentpack 是一个 AI-first 的本地“资产控制平面（asset control plane）”，用于管理与部署：
- `AGENTS.md` / instructions
- Agent Skills（`SKILL.md`）
- Claude Code slash commands（`.claude/commands`）
- Codex custom prompts（`~/.codex/prompts`）

## 为什么你可能需要 Agentpack

1) **跨工具一致性**
- 把同一套 agent 资产部署到多个工具（Codex、Claude Code、Cursor、VSCode…），通过显式 targets/mapping 保持一致。

2) **可复用 + 可回滚**
- 先 preview/diff，再 apply；通过 snapshots + rollback 与 manifest 安全删除，尽量避免误删/误覆盖用户文件。

3) **多机同步 + 可审计**
- 把 config repo 当成单一真相源，用 git 同步（`sync --rebase`），并在自动化场景依赖稳定契约（`--json` / MCP）。

## 一次完整闭环（从 update 到 rollback）

```bash
agentpack update
agentpack preview --diff
agentpack deploy --apply
agentpack status
agentpack rollback --to <snapshot_id>
```

说明：
- `deploy --apply` 与 `rollback` 都是写入类命令；自动化推荐用 `--json --yes`，并始终先跑 `preview`。
- `--json` 模式下，`deploy --apply` 会返回 `data.snapshot_id`，可直接传给 `rollback --to`。

## 演示（录制）

![Agentpack demo](docs/assets/demo.gif)

## 文档

- 文档入口：`docs/index.md`（英文）、`docs/zh-CN/index.md`（中文）
  - 英文： [docs/index.md](docs/index.md)
  - 中文： [docs/zh-CN/index.md](docs/zh-CN/index.md)
- 推荐从： [docs/tutorials/quickstart.md](docs/tutorials/quickstart.md)
- 5 分钟 demo（安全预览）： [docs/zh-CN/tutorials/demo-5min.md](docs/zh-CN/tutorials/demo-5min.md)
- “为什么不用 stow/chezmoi/yadm”： [docs/zh-CN/explanation/compare-dotfiles-managers.md](docs/zh-CN/explanation/compare-dotfiles-managers.md)
- 日常工作流： [docs/howto/workflows.md](docs/howto/workflows.md)
- CLI 参考： [docs/reference/cli.md](docs/reference/cli.md)
- Codex MCP 集成： [docs/howto/mcp.md](docs/howto/mcp.md)

## 为什么不用 dotfiles manager？

Agentpack 聚焦把 agent 资产部署到工具的可发现位置，并不试图管理你的整个 `$HOME`。

参考： [docs/zh-CN/explanation/compare-dotfiles-managers.md](docs/zh-CN/explanation/compare-dotfiles-managers.md)。

## 架构（高层）

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

更多细节见 `docs/zh-CN/explanation/architecture.md`。

## 安装

### Cargo

```bash
cargo install agentpack --locked
```

如果暂时无法从 crates.io 安装，可以从源码安装：

```bash
cargo install --git https://github.com/liqiongyu/agentpack --tag v0.9.1 --locked
```

### 预编译二进制

GitHub Releases: https://github.com/liqiongyu/agentpack/releases

## 快速开始

```bash
agentpack init
agentpack update
agentpack preview --diff
agentpack deploy --apply --yes
```

更完整的上手与阅读顺序见 `docs/index.md`；自动化用 `--json`（契约见 `docs/reference/json-api.md`）。

## 开发

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## 贡献

从 `AGENTS.md` 与 `CONTRIBUTING.md` 开始。
