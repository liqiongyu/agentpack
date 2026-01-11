# BACKLOG.md (v0.4 draft)

优先级：
- P0：v0.4 必须完成（平台化 & 合约固化）
- P1：v0.4 强烈建议（质量护城河）
- P2：可选（TUI/高级 evolve）
- P3：v0.5+（新增 targets）

## Milestone v0.4（P0/P1）

### Epic A：Docs / Contract 对齐（P0）
- [P0] A1. docs/ 作为最新文档；历史版本放 `docs/versions/`
- [P0] A2. README/PRD/ARCH/SPEC/BACKLOG 的版本标识一致（Current as of v0.4）
- [P0] A3. 版本号与 changelog 对齐（发布就绪）

### Epic B：Guardrails 全覆盖（P0）
- [P0] B1. 在代码中集中维护“写入命令集合”
- [P0] B2. `--json` + 无 `--yes` → `E_CONFIRM_REQUIRED`（覆盖 init/overlay edit 等边角命令）
- [P0] B3. guardrails tests 矩阵（每个写入命令至少 1 条）

### Epic C：TargetAdapter 平台化（P0）
- [P0] C1. 整理 targets 相关代码到 `targets/` 模块（减少 core/cli 中散落 if/else）
- [P0] C2. 固化 TargetAdapter 接口与 registry
- [P0] C3. core 统一 enforce 删除保护（manifest 规则）

### Epic D：AI-first 自描述能力（P0）
- [P0] D1. `agentpack help --json`（命令树 + 写入命令列表）
- [P0] D2. `agentpack schema`（JSON envelope + 关键命令 data 字段）
- [P0] D3. `preview --json --diff` 的 diff.files 结构 + size guard

### Epic E：Bootstrap 打磨（P0）
- [P0] E1. operator assets 覆盖 update/preview/status/explain/evolve propose
- [P0] E2. operator assets 写入版本标记，status 提示陈旧
- [P0] E3. bootstrap 行为与 guardrails 一致（json 必须 --yes）

### Epic F：Conformance Tests（P1）
- [P1] F1. conformance harness（对 codex/claude_code 跑通）
- [P1] F2. CI 中要求 conformance tests 通过（防回归）
- [P1] F3. 新增 target 的贡献者指南（TARGET_SDK / TARGET_CONFORMANCE）

## Milestone v0.5+（P3）
- 新 targets（Cursor/VS Code 等）：优先交给社区贡献或 v0.5+ 再做
- 更强 overlays（patch/3-way merge）
- registry/index 与模块发现
- eval gate + evolve apply（更自动化但仍可控）
