# Agentpack 文档（入口）

> Language: 简体中文 | [English](../index.md)

本页是 Agentpack 文档的唯一“从这里开始”入口。

## 选择你的路径

### 1) 从 0 开始（第一次使用）

- 按 Quickstart 完成安装、初始化配置仓库，并完成第一次部署：
  - `tutorials/quickstart.md`
- 了解日常闭环（update → preview → deploy → status → rollback）：
  - `howto/workflows.md`

### 2) 纳管已有资产（import）

- 如果你已经在磁盘上有 skills/prompts/commands，希望交给 Agentpack 管理：
  - CLI 参考：`reference/cli.md#import`
  - 工作流背景：`howto/workflows.md`

## 常用工作流

- 用 overlays 做本地定制（包括 patch overlays）：
  - `explanation/overlays.md`（见 `overlay edit --kind patch`）
  - `howto/overlays-create-sparse-materialize-rebase.md`
- “为什么不用 stow/chezmoi/yadm”（边界与适用场景）：
  - `explanation/compare-dotfiles-managers.md`
- 漂移 → 提案 → review → 合入：
  - `howto/workflows.md`
  - `howto/evolve.md`
- AI 自举（为 Codex / Claude Code 安装 operator assets）：
  - `howto/bootstrap.md`

## 自动化 / 集成

- 稳定的 `--json` 输出契约与示例：
  - `../reference/json-api.md`
  - `../reference/error-codes.md`
- Codex MCP 集成（`agentpack mcp serve`）：
  - `../howto/mcp.md`

## 参考

- CLI 命令参考：`reference/cli.md`
- 配置参考（`agentpack.yaml`）：`reference/config.md`
- Targets 参考：`reference/targets.md`
