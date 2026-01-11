# BACKLOG.md (v0.3 draft)

优先级：
- P0：v0.3 必须完成（减少摩擦 + 稳定性）
- P1：v0.3 强烈建议（AI-first 体验）
- P2：v1.0+（扩展生态与 targets）

## Milestone v0.2（已完成）

- manifest/lock/store
- overlays: upstream → global → machine → project
- plan/diff/deploy + snapshot/rollback
- per-root `.agentpack.manifest.json`
- remote set / sync
- bootstrap（operator assets）
- record/score/explain/evolve propose

## Milestone v0.3（P0/P1）

### Epic K：减少摩擦（命令链缩短）
- [P0] K1. `agentpack update` 组合命令（lock + fetch 编排）
  - [P0] K1.1 默认策略：lockfile 缺失时 lock+fetch；存在时默认 fetch
  - [P0] K1.2 --json 输出聚合 steps 信息
  - [P0] K1.3 `--json` 下要求 `--yes`
- [P1] K2. `agentpack preview` 组合命令（plan + 可选 diff）
  - [P1] K2.1 JSON 输出同时包含 plan/diff 摘要

### Epic L：Git checkout 缺失处理（消除脚枪）
- [P0] L1. 选择策略 A（推荐）：lockfile 存在但缓存缺失时自动 ensure_git_checkout
  - [P0] L1.1 仅对缺失 checkout 触发，不重复拉取
  - [P0] L1.2 错误信息清晰（网络不可用/权限问题）
- [P1] L2. （可选）提供 `--no-auto-fetch` 开关（给偏保守用户）

### Epic M：Overlay 编辑体验一致化
- [P0] M1. `overlay edit --scope global|machine|project`
  - [P0] M1.1 保持 `--project` 兼容（deprecated）
  - [P0] M1.2 machine scope 使用 `--machine` / 自动 machineId
- [P1] M2. `overlay path`（AI/脚本定位 overlay 路径）

### Epic N：写入安全规则强化（AI-first 保护栏）
- [P0] N1. 扩展 `--json` + `--yes` 保护到更多写命令
  - add/remove/lock/fetch/remote set/sync/record
  - 统一错误码：E_CONFIRM_REQUIRED
- [P1] N2. （可选）引入 `--interactive`（仅 human 模式）用于确认

### Epic O：Git hygiene（manifest 误提交风险）
- [P0] O1. doctor 增强：检测 `.agentpack.manifest.json` 是否位于 git repo 且未 ignore
- [P1] O2. `doctor --fix`：幂等写入 `.gitignore`（需要显式启用）

### Epic P：AI-first 闭环增强（模板与引导）
- [P1] P1. 更新 bootstrap 内置 Codex skill 文案：加入 record/score/explain/evolve
- [P1] P2. 增加 Claude slash command：/ap-evolve-propose（或在 /ap-status 中引导）

## Milestone v1.0（P2）

- 更多 targets adapters（Cursor/VS Code 等）
- registry/index 对接与模块发现
- overlay 3-way merge / patch overlays
- TUI（ratatui）
- 更完整的自动化进化（watcher、建议生成器等）
