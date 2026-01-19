# Agentpack 文档（入口）

> Language: 简体中文 | [English](../index.md)

本页是 Agentpack 文档的唯一“从这里开始”入口。

## 选择你的路径

### 1) 从 0 开始（第一次使用）

- 按 Quickstart 完成安装、初始化配置仓库，并完成第一次部署：
  - `../QUICKSTART.md`
- 了解日常闭环（update → preview → deploy → status → rollback）：
  - `../WORKFLOWS.md`

### 2) 纳管已有资产（import）

- 如果你已经在磁盘上有 skills/prompts/commands，希望交给 Agentpack 管理：
  - CLI 参考：`CLI.md#import`
  - 工作流背景：`../WORKFLOWS.md`

## 常用工作流

- 用 overlays 做本地定制（包括 patch overlays）：
  - `OVERLAYS.md`（见 `overlay edit --kind patch`）
- 漂移 → 提案 → review → 合入：
  - `../WORKFLOWS.md`
  - `EVOLVE.md`
- AI 自举（为 Codex / Claude Code 安装 operator assets）：
  - `../BOOTSTRAP.md`

## 自动化 / 集成

- 稳定的 `--json` 输出契约与示例：
  - `../JSON_API.md`
  - `../ERROR_CODES.md`
- Codex MCP 集成（`agentpack mcp serve`）：
  - `../MCP.md`

## 参考

- CLI 命令参考：`CLI.md`
- 配置参考（`agentpack.yaml`）：`../CONFIG.md`
- Targets 参考：`../TARGETS.md`
