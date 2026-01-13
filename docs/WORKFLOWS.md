# 日常工作流

这份文档给你一套“每天都能用”的固定套路。你可以把它当作最佳实践，也可以直接交给 Codex/Claude 让它按流程执行。

## 1) 最常用：更新 → 预览 → 应用

1. 更新依赖（锁文件缺失时会自动 lock）：
- `agentpack update`

2. 预览（建议总是带 diff）：
- `agentpack preview --diff`

3. 应用：
- `agentpack deploy --apply`

常用变体：
- 只对某个 profile：`agentpack --profile work update && agentpack --profile work preview --diff && agentpack --profile work deploy --apply`
- 只对某个 target：`agentpack --target codex preview --diff`

## 2) 多机器同步（把 config repo 当单一真源）

建议策略：配置 repo 走 git，同步时使用 rebase。

1. 设置 remote：
- `agentpack remote set <url>`

2. 同步：
- `agentpack sync --rebase`

冲突处理：
- agentpack 不会自动解决 merge 冲突。遇到冲突会失败并提示你用 git 手动解决。

## 3) 覆盖保护（adopt_update）与安全接管

当计划里出现 `adopt_update`：
- 代表目标路径已有文件，但它不在托管范围内（不是由 agentpack 之前写入/管理）。
- 默认 `deploy --apply` 会拒绝覆盖。

如果确认要接管（覆盖并纳入托管）：
- `agentpack deploy --apply --adopt`

推荐做法：
- 先 `agentpack preview --diff` 看清楚影响范围。
- 能用 overlays 解决的尽量用 overlays，避免覆盖用户手工文件。

## 4) 用 overlays 做本地定制（建议优先 sparse overlay）

典型场景：你加了一个上游 skill/command，但想加两句自己的说明或改默认行为。

1. 创建稀疏 overlay：
- `agentpack overlay edit <module_id> --sparse`

2. 在 overlay 目录里只放你改动的文件（保持最小差异）。

3. 重新 preview / deploy：
- `agentpack preview --diff`
- `agentpack deploy --apply`

上游更新后（你跑了 `update` 拉到新 commit）：
- `agentpack overlay rebase <module_id> --sparsify`

如果 rebase 发生冲突：
- 会返回 `E_OVERLAY_REBASE_CONFLICT`，并列出冲突文件。
- 打开 overlay 目录里带冲突标记的文件，手动解决后再跑一次 rebase 或直接 commit。

## 5) 漂移（status）→ 提案（evolve propose）→ review → 合入

目标：把“线上（目标目录）里的改动”回流到 config repo，变成可 review 的 overlay 改动。

1. 检查漂移：
- `agentpack status`

2. 生成提案（建议先 dry-run 看候选）：
- `agentpack evolve propose --dry-run --json`

3. 确认没问题后创建提案分支（会在 config repo 创建 branch 并写入 overlay 文件）：
- `agentpack evolve propose --scope global`

4. 进入 config repo review：
- `cd ~/.agentpack/repo && git status && git diff`
- 提交、开 PR（如果你用远端）或本地合入。

5. 最后重新部署，使目标目录与期望态一致：
- `agentpack deploy --apply`

说明：
- 默认只会对“能安全映射回单个模块”的 drift 生成 proposal。
- 对聚合输出（例如 Codex 的 `AGENTS.md`）如果包含模块分段 marker，agentpack 可以把 drift 映射回具体 instructions 模块片段。

## 6) 恢复缺失文件（evolve restore）

当 status 显示一些托管文件被删了（missing），你只想“把缺失的创建回来”，不想更新/删除其他东西：

- 预览：`agentpack evolve restore --dry-run --json`
- 应用：`agentpack evolve restore`

特性：
- create-only：只创建缺失文件，不更新已存在文件，不删除任何文件。

## 7) 自动化/Agent 调用建议（--json）

如果你把 agentpack 当作可编排组件（例如让 Codex CLI 调用）：
- 读取 `--json` 输出做下一步决策。
- 遇到 `E_CONFIRM_REQUIRED`：必须在确认写入意图后加 `--yes` 重试。
- 遇到 `E_ADOPT_CONFIRM_REQUIRED`：必须明确加 `--adopt` 才能覆盖非托管文件。

推荐模式：
- 只要是写入类命令（deploy --apply / update / lock / fetch / bootstrap / evolve propose/restore / overlay edit/rebase / rollback 等），自动化里总是显式加 `--yes`，并在执行前先 `preview`。

更多：`JSON_API.md`、`ERROR_CODES.md`。
