# PRD.md (v0.3 draft)

## 1. 背景与问题

你会同时使用多个 AI coding 工具（Codex CLI、Claude Code、Cursor 等），而这些工具又各自支持一套「可插拔资产」：AGENTS.md、skills、slash commands、prompts、subagents、/prompts、commands 等。

这些资产带来的价值很大（效率与一致性），但现实里管理成本也很高：
- 发现成本：同一类能力（例如 git review）可以有上百种实现，你需要搜索、理解、试用、筛选。
- 维护成本：安装后会不断「微调」以适配自己的习惯与项目；上游更新后你还要合并。
- 协同成本：多台电脑、多项目、多工具，版本不一致与路径差异会带来大量摩擦。
- 可控性：想要可审计、可回滚、可复现，而不是“我也不知道现在生效的是哪个版本”。

Agentpack 的定位是：在本地提供一个“资产控制平面（control plane）”，用声明式清单 + overlays + lockfile，把资产从「管理」到「分发」再到「回滚」统一起来，并对 AI/人类都友好（AI-first）。

## 2. 目标用户

- 深度 AI coding 用户：日常高频使用 Codex CLI/Claude Code 等，愿意把 prompts/skills 当作生产资料。
- 多机器、多项目开发者：需要一致的基线 + 项目差异化。
- 对可审计、可回滚、可复现有要求：不希望资产管理变成不可控的“脚本+提示词地狱”。

## 3. v0.2 已实现能力（现状）

v0.2 已经覆盖了核心闭环：
- 单一真源 Config Repo（git 管理）：manifest + overlays
- lockfile（agentpack.lock.json）：固定 git modules 的 commit + sha
- store/cache：拉取 git sources 并校验 hash
- overlays 分层：upstream → global → machine → project
- plan/diff/deploy：生成与写入真实文件（copy/render），支持备份、快照与 rollback
- 目标目录托管清单：每个 target root 写入 `.agentpack.manifest.json`，用于安全删除与可靠 drift/status
- 多机器同步：remote set / sync
- AI-first 自举：bootstrap 安装 Codex operator skill + Claude slash commands
- 最小进化闭环：record/score + explain + evolve propose（把 drift 捕获成 overlay proposal 分支）

## 4. v0.3 需要解决的新增痛点

虽然 v0.2 已可用，但从“日常深度使用”的角度看，还存在几类明显摩擦：

1) 命令链太长：
- lock + fetch + plan + diff + deploy… 对人类和 AI 都有学习成本。

2) 依赖预热的脚枪（sharp edges）：
- lockfile 存在但 store 缓存缺失时，plan/materialize 可能报错；用户必须记得先 fetch。

3) overlays 的编辑体验还不够顺滑：
- 已有 global/project overlay edit，但 machine overlay 缺少同等体验（需要手工建目录）。

4) “托管 manifest”可能污染项目仓库：
- 某些 target root 是项目 repo root 或其子目录，`.agentpack.manifest.json` 有误提交风险。

5) AI-first 的闭环还不够“顺手”：
- v0.2 的 evolve propose 能把 drift 变成 overlay，但 operator assets 还可以更明确地引导 AI 走这条最佳路径。

## 5. v0.3 产品目标

P0（必须做到）：
- 减少常见路径的命令数量，让“更新依赖/预览/应用”更接近一两个命令。
- 消除 lockfile+缓存缺失导致的常见报错，要么自动补齐（安全网络操作），要么给出明确可操作的错误信息。
- overlays 编辑体验覆盖 global/machine/project 三种 scope。
- 降低 `.agentpack.manifest.json` 被误提交的概率（至少做到明确告警 + 可选自动修复）。

P1（强烈建议做到）：
- 为 AI 输出更“可直接拿去做下一步”的结构化信息（例如 update/preview 的 JSON 输出更聚合）。
- operator assets 升级：包含 record / evolve propose 的推荐流程。

## 6. 需求范围（v0.3 In-scope）

- 新增组合命令（Composite Commands）：
  - `agentpack update`：一键 lock+fetch（并可选只做其一），输出清晰的结果与错误提示。
  - `agentpack preview`：一键 plan(+可选 diff)，便于 AI 工具调用。

- 依赖缺失处理：
  - 当 lockfile 指向的 git checkout 在 store 中缺失时，提供：
    - 默认自动补齐（推荐）；或
    - 明确错误提示“缺少缓存，请运行 agentpack fetch/agentpack update”。

- overlays 编辑体验增强：
  - `agentpack overlay edit --scope global|machine|project`（取代/兼容 `--project`）。
  - （可选）`agentpack overlay path`：把 overlay 目录路径输出给 AI 或脚本。

- Git hygiene：
  - `agentpack doctor` 增加“manifest file 在 git repo 内是否被 ignore”的告警。
  - （可选）`agentpack doctor --fix`：自动写入 `.gitignore`（默认不启用，需要显式用户同意）。

- AI-first 资产升级：
  - 更新 bootstrap 模板：让 Codex/Claude 的 operator 更自然地使用 record/score/explain/evolve。

## 7. 非目标（v0.3 Out-of-scope）

- MCP 全量生态管理（可先降低优先级）。
- 复杂三方合并（3-way merge）与 patch overlays（可以在 v1.0 做）。
- GUI（先保持 CLI + 可选 TUI）。
- 远程 registry/marketplace 的完整发现体系（先以 git/local/手动为主，后续再做）。

## 8. 成功指标（建议）

- 日常使用的“平均命令长度”降低（从 4~6 个命令 → 1~2 个命令完成一次更新与预览）。
- 新用户从安装到首次成功 deploy 的时间下降。
- 因缓存缺失导致的报错显著降低。
- evolve propose 的使用率上升（AI/人类更容易把改动沉淀为 overlays）。

## 9. 里程碑

- v0.2：已完成（稳定闭环）
- v0.3：以“减少摩擦 + 强化 AI-first 闭环”为主
- v1.0：扩展更多 targets（Cursor/VS Code），更强 overlays，registry 对接
