# ARCHITECTURE.md (v0.4 draft)

## 1. v0.4 架构目标

v0.4 的目标不是“加更多功能”，而是把 v0.3 的实现平台化：
- core/targets 解耦更清晰
- 契约（contract）更稳定
- 回归测试更系统化（conformance tests）

## 2. 分层建议（不强制拆 crate，但建议按目录/模块清晰分层）

- core（核心管道）
  - config / lockfile / sources / overlays
  - planner / differ
  - apply / snapshot / rollback
  - manifest（ownership/delete 的核心语义在这里 enforce）
  - json_contract（envelope + error codes + schema 输出）

- targets（目标适配层）
  - codex adapter
  - claude_code adapter
  - common helpers（root 定义、ignore 规则）
  - conformance test harness

- cli
  - 参数解析
  - 调用 core/targets
  - 输出 human 或 JSON

## 3. TargetAdapter 边界（v0.4 要钉死）

目标：新增 target 时，贡献者只需要实现“映射规则 + roots + 校验”，不用理解 core 细节。

建议的 TargetAdapter 最小接口（概念）：
- detect(ctx) -> DetectReport
- roots(ctx) -> Vec<Root>
- map(desired_state, ctx) -> MappedOutputs
- validate(mapped_outputs, ctx) -> ValidationReport

核心语义由 core enforce：
- delete 只能针对 manifest 管控的 path
- 没有 manifest 时，delete 必须跳过，并输出 warning
- apply 必须写 per-root manifest
- status/drift 基于 manifest 判定 missing/modified/extra

## 4. Conformance Tests（v0.4 的质量护城河）

新增/重构目标：
- 一套通用测试，任何 target 都必须通过（至少 codex、claude_code）
- 重点锁定“危险语义”：
  1) 删除保护（ownership）
  2) manifest 写入与读取
  3) drift 分类（missing/modified/extra）
  4) rollback 可恢复 create/update/delete
  5) JSON 合约（schema_version、错误码）

建议做法：
- 以临时目录模拟 target roots
- 通过真实管道跑 plan/apply/status
- 用 golden tests 固化行为（尤其 plan 的排序/解释字段）

## 5. AI-first 的平台化

v0.4 建议把“AI-first”从文案变成接口能力：
- `agentpack help --json`：输出命令树、参数、哪些是写命令（需 --yes）
- `agentpack schema`：输出 JSON envelope 合约与关键命令 data 的字段定义
- `preview --json --diff`：输出结构化 diff.files（让 AI 直接用于下一步判断/解释/生成 PR 描述）

operator assets（bootstrap）要以这些稳定接口为基础，让 AI 更“闭环”。
