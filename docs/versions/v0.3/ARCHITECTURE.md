# ARCHITECTURE.md (v0.3 draft)

## 1. 总览

Agentpack = CLI（AI-first） + Core Engine + Target Adapters +（可选）TUI

它的本质仍然是“声明式资产编译器（declarative asset compiler）”：
- 输入：manifest（想要什么） + overlays（怎么改） + lockfile（锁版本）
- 输出：写入各工具“可发现”的目录/配置文件（真实文件，copy/render）
- 闭环：plan → diff → apply → snapshot → rollback

v0.3 的架构演进重点不是引入更多复杂模块，而是把“常用路径”做成更短、更稳、更适合 AI 调用的工作流。

## 2. 三层存储模型（保持不变）

A) Config Repo（Git 管理，建议同步到远端）
- 内容：agentpack.yaml + overlays (+ 可选 modules)
- 目标：可 review、可 PR、可回滚

B) Cache/Store（不进 Git）
- 内容：git checkouts、hash 校验、中间产物
- 目标：可复现；允许丢弃后重建

C) Deployed Outputs（不进 Git）
- 写入到目标工具目录的“生效形态”
- v0.2+：每个 target root 写入 `.agentpack.manifest.json`（托管清单）

## 3. 目录布局（默认）

AGENTPACK_HOME（默认）：`~/.agentpack`（可通过 `AGENTPACK_HOME` 覆盖）

- repo/
  - agentpack.yaml
  - agentpack.lock.json
  - modules/
  - overlays/
  - overlays/machines/<machineId>/
  - projects/<projectId>/overlays/
- cache/
  - git/<moduleId>/<commit>/...
- state/
  - snapshots/<id>.json
  - snapshots/<id>/backup/...
  - logs/events.jsonl

## 4. 关键组件

### 4.1 CLI Frontend（AI-first）
- 所有核心命令支持 `--json`
- JSON 输出使用 envelope：schema_version / ok / command / version / data / warnings / errors
- 对写入磁盘或改写 git 的命令：在 `--json` 下倾向要求 `--yes`（可配置/逐步收紧）

v0.3 新增：组合命令（Composite Commands）
- `agentpack update`：把 lock+fetch 变成一个可复用原子动作
- `agentpack preview`：把 plan(+diff) 变成一个可复用动作

这些组合命令本质上是“编排层（orchestrator）”，复用已有子命令能力，而不是重复实现。

### 4.2 Config Loader
- 读取 agentpack.yaml
- 读取 lockfile（如果存在）
- 读取项目上下文（cwd、git root、origin url）
- 读取 machineId

### 4.3 Resolver + Lock
- 将 source（local/git）解析为具体版本
- 产出 lockfile（commit + sha256 + file manifest）

### 4.4 Cache Manager（Store）
- `ensure_git_checkout()` 拉取并缓存第三方模块
- `hash_tree()` 校验可复现

v0.3 建议增强：
- 当引擎需要某个 checkout 而本地缓存缺失时：
  - 要么自动补齐（安全网络操作，默认可开）；
  - 要么输出更清晰的错误信息（提示运行 update/fetch）。

### 4.5 Overlay Engine
- 四层覆盖（优先级从低到高）：
  1) upstream module
  2) global overlay
  3) machine overlay
  4) project overlay

v0.3 建议增强：
- `overlay edit` 的 scope 一致化：global/machine/project 都能一键生成 skeleton + 打开编辑器

### 4.6 Renderer/Compiler
- 将合成后的 module 编译为 target 所需的目录结构
- v0.2/0.3 默认不做模板变量渲染（仅组合/复制内容）
- 输出为 DesiredState（target+path→bytes+module_ids）

### 4.7 Target Adapters
- 负责：路径选择、输出布局、root 列表（用于 manifest 与 drift 扫描策略）
- v0.2+：每个 root 写入 `.agentpack.manifest.json`

### 4.8 Apply + Snapshot + Rollback
- apply：原子写入 + 备份
- snapshot：记录变更与备份位置
- rollback：基于 snapshot 恢复

### 4.9 Observability + Evolution
- record：写入 events.jsonl
- score：模块健康度（失败率、调用量等）
- explain：plan/diff/status 的来源解释（upstream/global/machine/project）
- evolve propose：把 drift 捕获为 overlay proposal 分支（可 review + merge）

v0.3 建议增强：
- bootstrap 的 operator assets 明确推荐“编辑 deployed → evolve propose → review/merge”的路径

## 5. Git Hygiene（v0.3 新增关注点）

`.agentpack.manifest.json` 写入位置属于 target root。
当某个 root 位于项目 git repo 内时，会存在误提交风险。

v0.3 推荐策略：
- doctor 提示风险（发现 manifest 位于 git repo 且未被 ignore）。
- 可选 `doctor --fix`：在用户明确同意的前提下写入 `.gitignore`。

## 6. 扩展性

- 新 target adapter 是 v1.0 重点（Cursor/VS Code 等）。
- 不建议在 v0.3 引入 GUI；优先把 CLI 做成“人类可用 + AI 可编排”。
