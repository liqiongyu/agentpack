# README.md (v0.4 plan)

v0.4 的主题：**把基础打扎实**（contract + adapter 抽象 + conformance tests + operator UX），而不是急着加 targets。

## v0.3 已具备的能力（基线）
- config repo + lockfile + cache
- overlays：upstream → global → machine → project
- plan/diff/deploy + snapshot/rollback
- per-root manifest（安全删除 + drift/status）
- remote/sync（多机器）
- AI-first：--json + guardrails + bootstrap operator assets
- evolve：record/score/explain/evolve propose（提案式）

## v0.4 要新增/固化的能力
- 合约固化：错误码/JSON schema/版本策略
- 自描述：help --json / schema
- preview JSON diff 增强（diff.files）
- TargetAdapter 平台化 + conformance tests
- operator assets 版本标记与“过期提示”

## 为什么 v0.4 不加 targets？
- targets 增加不会改变核心价值，但会显著增加维护与回归成本。
- v0.4 先把 adapter 贡献路径铺好，后续 targets 更适合交给社区 contributor。
