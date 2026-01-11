# SPEC.md (v0.3 draft)

说明：本规范面向实现者（Rust + clap），用于指导 v0.3 的增量开发。

本 spec 以 v0.2 已有行为为基础，只定义 v0.3 的新增/变更点。

## 1. CLI 总则

### 1.1 全局参数（沿用 v0.2）

- `--repo <path>`：配置仓库目录（默认 `$AGENTPACK_HOME/repo`）
- `--profile <name>`：profile（默认 `default`）
- `--target <all|codex|claude_code>`：目标过滤（默认 `all`）
- `--machine <id>`：覆盖 machineId（默认自动检测/环境变量）
- `--json`：输出 JSON envelope
- `--yes`：确认执行写入（主要用于 `--json` 场景）
- `--dry-run`：强制不写入（即使显式 apply）

### 1.2 写入安全规则（v0.3 强化）

v0.2 已对 `deploy --apply`、`bootstrap --apply`、`evolve propose` 在 `--json` 下要求 `--yes`。

v0.3 建议把同一规则扩展到“任何会写磁盘/改写 git 的命令”，至少包括：
- `add`, `remove`, `lock`, `fetch`, `remote set`, `sync`, `record`（写日志也算写入）

推荐策略：
- 当 `--json` 且命令会写入时，如果缺少 `--yes`：
  - 直接报错（machine-friendly），错误码建议 `E_CONFIRM_REQUIRED`。
- 人类输出模式（无 `--json`）下：
  - 允许交互式确认（y/N）或沿用现状（尽量不引入交互，除非明确需要）。

## 2. 新增组合命令（Composite Commands）

### 2.1 `agentpack update`

目标：把常见的“锁版本 + 拉取依赖”合并为一个命令，减少日常摩擦。

命令：
- `agentpack update [--lock] [--fetch] [--no-lock] [--no-fetch]`

默认行为（建议）：
- 如果 lockfile 不存在：默认执行 lock + fetch
- 如果 lockfile 已存在：默认只 fetch（更快），除非显式 `--lock`

参数建议：
- `--lock`：强制重新生成 lockfile
- `--fetch`：强制执行 fetch
- `--no-lock`：明确禁用 lock
- `--no-fetch`：明确禁用 fetch

输出：
- human：显示执行了哪些步骤、耗时、拉取了多少模块
- json：
  - `data.steps`: [{name, ok, detail}]
  - `data.lockfile_path`
  - `data.store_path`
  - `data.git_modules_fetched`
  - `warnings[]`

注意：
- `update` 本质是 orchestrator，内部复用 lock/fetch 的逻辑，避免代码重复。
- `--json` 下属于写入命令，需要 `--yes`（遵循 1.2）。

### 2.2 `agentpack preview`

目标：把“plan +（可选）diff”合并为一个 AI 友好的预览命令。

命令：
- `agentpack preview [--diff]`

行为：
- 总是运行 plan
- 当 `--diff` 时再运行 diff（human 输出为 unified diff；json 输出为 diff 摘要即可）

输出（json）：
- `data.plan.summary` + `data.plan.changes`
- 如果 `--diff`：`data.diff.summary` + `data.diff.changes`
- `warnings[]`

备注：
- preview 是纯读取操作，不需要 `--yes`。

## 3. Git source 缓存缺失处理（v0.3 行为变更）

问题：当 lockfile 存在且 engine 优先使用 lockfile 指向的 checkout 时，如果本地 cache 缺失，materialize 会报错。

v0.3 需要二选一（推荐 A）：

A) 自动补齐（推荐默认）
- 当 lockfile 解析到 `<moduleId, commit>` 时：
  - 如果 checkout_dir 不存在，则自动执行 `ensure_git_checkout()`。
- 这属于安全网络操作（只读拉取），不会写入 target outputs。

B) 明确报错（保守默认）
- 当 checkout_dir 不存在：
  - 报错：`missing git checkout for module <id> at <commit>; run agentpack fetch/agentpack update`

无论选择 A 或 B，都应保证错误信息可操作、对 AI 友好。

## 4. Overlay 编辑体验一致化（v0.3）

### 4.1 `agentpack overlay edit --scope`

v0.2：
- `agentpack overlay edit <module_id> [--project]`

v0.3：
- 新增 `--scope <global|machine|project>`（默认 global）
- `--project` 仍兼容，但标记为 deprecated（内部映射到 `--scope project`）

scope 行为：
- global: `repo/overlays/<moduleId>/...`
- machine: `repo/overlays/machines/<machineId>/<moduleId>/...`
- project: `repo/projects/<projectId>/overlays/<moduleId>/...`

编辑行为：
- overlay 不存在时：复制 upstream → overlay_dir，写入 `.agentpack/baseline.json`
- overlay 已存在时：不覆盖，只确保 baseline 存在
- 如果设置了 `$EDITOR`：打开 overlay_dir

json 输出：
- `data.scope`
- `data.module_id`
- `data.overlay_dir`
- `data.created`
- `data.machine_id`（当 scope=machine）
- `data.project_id`（当 scope=project）

### 4.2 `agentpack overlay path`（可选但很实用）

命令：
- `agentpack overlay path <module_id> --scope <...>`

用途：
- 让 AI/脚本不需要理解目录布局，也能定位要写入的 overlay 路径。

输出：
- human：打印绝对路径
- json：`data.overlay_dir`

## 5. Git Hygiene（v0.3）

### 5.1 doctor 增强

当 target manifest `.agentpack.manifest.json` 位于某个 git repo 内时：
- doctor 输出 warning：
  - 该文件可能被误提交
  - 建议把它加入 `.gitignore`

### 5.2 `doctor --fix`（可选）

命令：
- `agentpack doctor --fix`（默认不开；需要用户明确使用）

行为：
- 在对应 repo 的 `.gitignore` 追加一行：`.agentpack.manifest.json`
- 必须做到幂等：重复执行不会重复插入

安全：
- `--json` 下需要 `--yes`

## 6. bootstrap 模板升级（v0.3）

v0.3 建议更新内置 operator assets：
- Codex skill：增加“如何使用 record/score/explain/evolve propose”的推荐流程
- Claude commands：增加 /ap-evolve-propose 或在现有命令说明里加入 evolve 提示

模板更新属于 repo 内置 assets，不影响 core 行为，但能显著提升“AI-first 闭环”。
