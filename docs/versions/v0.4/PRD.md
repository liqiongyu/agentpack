# PRD.md

> Current as of **v0.4 (draft)** (2026-01-11). Historical snapshots live under `docs/versions/`.

## 1. 背景与问题

v0.3 已经把 Agentpack 的核心闭环跑通（可用、可回滚、可同步、AI-first）：
- 单一真源（config repo）+ lockfile + cache
- overlays（upstream → global → machine → project）
- plan/diff/deploy + snapshot/rollback
- per-root `.agentpack.manifest.json`（安全删除 + drift/status）
- remote/sync（多机器一致性）
- AI-first：`--json` 合约、guardrails、bootstrap（operator assets）
- evolve：record/score/explain/evolve propose（提案式进化）

因此 v0.4 **不急着新增 targets**。加 targets 永远有价值，但它更适合作为：
- v0.5+ 的官方扩展；或
- 社区 contributor 的增量贡献。

v0.4 更重要的是把“平台基础”与 “AI-first bootstrap” 打磨到足够稳定，降低后续扩展成本与回归风险。

## 2. v0.4 的主题：Platformize（平台化）

v0.4 要解决三类“长期稳定性问题”：

1) Contract 稳定
- JSON 输出与错误码要足够稳定，AI 才能可靠编排。
- schema/版本/兼容策略要明确，避免每次 refactor 都把上层工具打碎。

2) Targets 抽象更清晰
- 新增 target 不应需要修改一堆核心逻辑（engine/cli 里到处 if/else）。
- 需要明确的 TargetAdapter 边界 + 一套 conformance tests（语义回归）。

3) Operator assets（bootstrap）更“顺手”
- 让 Codex/Claude 的 operator assets 不只是“能调用 agentpack”，而是能引导最佳实践路径：
  - doctor → update → preview → (explain) → deploy
  - status → evolve propose → review → deploy
- 同时要更安全：默认只读/预览，apply 必须显式确认。

## 3. v0.4 目标

### P0（必须）
A. Contract & Compatibility
- 明确并固化：JSON envelope schema_version、错误码枚举、manifest/lockfile 的版本策略
- `--json` 下所有“写入命令”统一要求 `--yes`（含 init、overlay edit 等边角命令）
- `preview --json --diff` 输出结构增强（给 AI 直接消费）

B. TargetAdapter 抽象与贡献者路径
- 整理代码结构：targets 的路径/布局/校验集中在一个模块
- 提供 Target SDK 文档与 conformance tests（至少 codex/claude_code 先跑通）
- CI 中引入 conformance tests（防回归）

C. AI-first bootstrap 打磨
- operator assets 覆盖 update/preview/status/explain/evolve propose 的推荐路径
- operator assets 写入版本标记，status 可提示“operator 已过期，需要 bootstrap 更新”
- 提供自描述能力：`agentpack help --json` / `agentpack schema`（让 AI 学会怎么用）

### P1（强烈建议）
- `agentpack migrate`（仅当 schema 真的发生变化时；默认不需要）
- 可选 TUI（只读）：profile/modules/targets 列表、preview/status 视图

## 4. 非目标（v0.4 Out-of-scope）

- 新增新的 targets（Cursor/VS Code 等）
- MCP 全量生态管理（保持低优先级）
- GUI
- 自动无监督“自我进化直接落地”（依旧保持 proposal → review → deploy）

## 5. 用户旅程（v0.4 期望更顺滑）

Journey 1：新机器初始化（人/AI）
1) `agentpack init`（或 init --clone）
2) `agentpack doctor --fix`
3) `agentpack update`
4) `agentpack preview --diff`
5) `agentpack deploy --apply`

Journey 2：项目定制（overlay）
1) `agentpack overlay edit --scope project`
2) `agentpack preview --project <path> --diff`
3) `agentpack deploy --apply`

Journey 3：提案式进化（evolve propose）
1) `agentpack status`
2) `agentpack evolve propose --module <id>`
3) review diff /（可选）eval gate
4) `agentpack deploy --apply`

## 6. 成功指标（v0.4）

- 新增一个 target 的 PR：大部分改动只在 `targets/` 与 conformance tests，不需要理解 core 全部细节
- AI 编排可靠性：AI 基于 `help --json` + `schema` + `preview --json --diff` 可稳定执行部署
- 回归率下降：核心语义由 conformance tests 锁住

## 7. 里程碑

- v0.4.0：contract 固化 + adapter 抽象 + bootstrap 打磨 + conformance tests 初版
- v0.4.1：conformance tests 完善 + docs/README 对齐 + 可选 migrate
- v0.5.0：新增 targets（可由社区贡献优先）
