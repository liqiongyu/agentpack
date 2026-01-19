# Agentpack Docs

> Language: 简体中文 | [English](../README.md)

这份文档集面向两类读者：
- **用户（会用就行）**：照着做能完成安装、配置、预览、部署、回滚，并把改动沉淀为 overlays。
- **贡献者（要改代码/加 target）**：需要理解引擎、数据模型、`--json` 契约、conformance 测试。

英文文档（`docs/`）为权威版本；简体中文用户文档位于 `docs/zh-CN/`，可能会有少量滞后。

如果你只想“现在就用起来”，从《快速开始》开始。

## 用户文档（推荐阅读顺序）

1) **快速开始**：`QUICKSTART.md`
- 30 分钟从 0 到第一次 deploy

2) **日常工作流**：`WORKFLOWS.md`
- 更新依赖（update）→ 预览（preview）→ 应用（deploy --apply）
- 漂移（status）→ 提案（evolve propose）→ review → 合入

3) **CLI 命令参考**：`CLI.md`
- 全局参数、每个命令的用途/关键 flag/示例

4) **配置文件与模块**：`CONFIG.md`
- `agentpack.yaml`（profiles/targets/modules）与 source spec（local/git）

5) **Targets（Codex / Claude Code）**：`TARGETS.md`
- 写入位置、options、限制（尤其是 prompts 仅 user scope）

6) **Overlays**：`OVERLAYS.md`
- global/machine/project 三层 overlay
- `overlay edit --sparse/--materialize` 与 `overlay rebase`（3-way merge）
- patch overlays：`overlay edit --kind patch`

7) **AI 自举（Bootstrap）**：`BOOTSTRAP.md`
- 安装 operator assets：让 AI 自己会用 agentpack

8) **Evolve（自进化闭环）**：`EVOLVE.md`
- record/score/explain/propose/restore

9) **排障**：`TROUBLESHOOTING.md`
- 常见错误码、冲突、权限、Windows 路径问题等

10) **MCP（Codex 集成）**：`../MCP.md`
- 在 Codex 中把 `agentpack mcp serve` 配置成 MCP server。

## 参考/契约（给自动化与贡献者）

- **SPEC（实现对齐的唯一权威）**：`SPEC.md`
- **JSON 输出契约**：`JSON_API.md`
- **稳定错误码注册表**：`ERROR_CODES.md`
- **架构总览**：`ARCHITECTURE.md`
- **Target SDK（如何加新 target）**：`TARGET_SDK.md`
- **Target conformance**：`TARGET_CONFORMANCE.md`

## 维护者/工程化

- 发布流程：`RELEASING.md`
- 依赖与安全检查：`SECURITY_CHECKS.md`
- 治理层（opt-in）：`../GOVERNANCE.md`（包含 `agentpack policy lint|lock`）
- 产品文档（历史背景/规划）：`PRD.md`、`BACKLOG.md`
- Codex 执行指南（唯一 active）：`../dev/codex.md`

## 外部参考（上游工具的官方文档）

- AGENTS.md（规范/示例）：https://agents.md/
- Codex：
  - AGENTS.md 指令发现：https://developers.openai.com/codex/guides/agents-md/
  - Skills：https://developers.openai.com/codex/skills/
  - Custom Prompts：https://developers.openai.com/codex/custom-prompts/
- Claude Code：
  - Slash commands：https://code.claude.com/docs/en/slash-commands
