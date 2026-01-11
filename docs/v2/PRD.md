# PRD.md (v0.2)

## 1. 背景与问题

AI coding 工具生态正在快速分化（Claude Code / Codex CLI / Cursor / 各类 IDE Agents）。用户在不同工具、不同项目、不同机器上需要复用并持续维护一组“Agent 资产”：

- instructions（如 AGENTS.md / project instructions）
- prompts（如 codex prompts）
- commands（如 Claude slash commands / 自定义命令）
- skills（如 SKILL.md + 脚本）
- 可选：MCP（初期降级优先级）

现实痛点：
- 资产来源分散、安装方式不同、升级/回滚困难
- 多项目与多机器之间版本漂移
- 用户对同一个资产会不断手动微调（“进化”需求）
- 工具目录与规范不同，复制粘贴与手工维护成本高且易错

Agentpack 的目标是成为一个本地的“资产控制平面”，用统一抽象管理这些资产，并可安全分发到不同 coding agent 工具及项目中。

## 2. v0.2 目标

### 2.1 产品目标
1) 多机器一致性：同一份资产仓库在多台机器上可一键同步，减少版本漂移。
2) 多项目/多层 overlay：支持 global / machine / project 三层覆盖，且可解释、可预测。
3) 部署安全：deploy 不会误删用户文件；rollback 可信。
4) AI-first 可用：提供稳定的 JSON 输出与自检能力，方便 AI 工具调用 agentpack 完成闭环。
5) 进化的“可观测-可评估-可提案”最小闭环：先记录与评分，再生成可 review 的 patch 提案。

### 2.2 非目标（v0.2 不做）
- 完整的 MCP registry 管理器（只做基础 install/enable/disable 占位或保持现状）
- GUI（保留 CLI + 可选 TUI；GUI 留到生态成熟后）
- 自动无监督地改写用户资产并直接部署到生产（必须经过 diff/review）

## 3. 用户画像与使用场景

### 3.1 主要用户
- 深度 AI coding 用户：多工具、多项目、多机器，依赖 prompts/commands/skills
- 团队 Tech Lead：希望团队共享一套标准化 Agent 资产，并允许项目级定制

### 3.2 核心场景（v0.2）
- S1：新机器初始化（clone 同一份 agentpack repo，一键 bootstrap + deploy）
- S2：新项目接入（继承 global defaults，加 project overlay）
- S3：资产升级/回滚（更新上游 git source，lock/fetch，diff，deploy，必要时 rollback）
- S4：漂移检测（有人手动改了目标目录里的文件，status 能发现并解释）
- S5：进化提案（某个 command/skill 经常失败，系统能基于记录给出可 review 的 patch）

## 4. 关键体验原则

- 可预测：overlay 优先级清晰，plan 输出可解释
- 可回滚：每次 deploy 都可生成 snapshot，并可恢复
- 安全：只操作自己管理的文件（manifest）
- AI 与人一致：AI 调用与人操作走同一套命令与计划/差异模型
- 降低心智负担：默认路径最短（init → sync → plan → deploy）

## 5. 成功指标（v0.2）

- 从空机器到可用：<= 5 条命令完成 bootstrap + deploy（不含安装二进制）
- 99% 情况下 deploy 不会误删用户非托管文件（通过 manifest 机制保障）
- status 输出能定位：哪些文件漂移、来自哪个 overlay/module、预期是什么
- evolve propose 能产出可 review 的 patch（diff 可读），且不会破坏现有 deploy 流程

## 6. 风险与对策

- 风险：多机器同步冲突（git conflicts）
  - 对策：提供 agentpack sync 规范化 pull/rebase/push，冲突时给出清晰提示与建议
- 风险：进化功能导致用户不信任
  - 对策：v0.2 只做 propose，不做自动 apply；所有改动走 diff + snapshot

## 7. 版本规划

- v0.1：已完成基础命令闭环（init/add/lock/fetch/plan/diff/deploy/status/rollback/bootstrap）
- v0.2：remote/sync、manifest 安全、machine overlay、JSON 输出、doctor、record/score/propose
- v0.3：可选 MCP 支持增强、更多 targets（Cursor 等）、可选 TUI、半自动 evolve apply（带守护）
