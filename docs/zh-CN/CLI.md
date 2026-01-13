# CLI 参考

> Language: 简体中文 | [English](../CLI.md)

本文件面向“想快速查命令怎么用”的场景。更偏工作流的内容见：`WORKFLOWS.md`。

## 全局参数（所有命令都支持）

- `--repo <path>`：指定 config repo 路径（默认 `$AGENTPACK_HOME/repo`）
- `--profile <name>`：选择 profile（默认 `default`）
- `--target <codex|claude_code|cursor|vscode|all>`：选择 target（默认 `all`）
- `--machine <id>`：覆盖 machineId（用于 machine overlays；默认自动探测）
- `--json`：stdout 输出机器可读 JSON（envelope）
- `--yes`：跳过确认（注意：`--json` 下写入类命令必须显式给）
- `--dry-run`：强制 dry-run（即使传了 `deploy --apply` 或 `overlay rebase` 等也不写入）

提示：
- `agentpack help --json` 会返回结构化的命令列表与 mutating 命令集合。
- `agentpack schema --json` 会返回 JSON envelope 与常用命令的 data 字段提示。

## init

`agentpack init [--git] [--bootstrap]`
- 初始化 config repo skeleton（创建 `agentpack.yaml` 与示例目录）
- 默认不会自动 `git init`
- `--git`：同时初始化 git repo，并确保 `.gitignore` 忽略 `.agentpack.manifest.json`
- `--bootstrap`：同时安装 operator assets 到 config repo（等价于执行 `agentpack bootstrap --scope project`）

## add / remove

- `agentpack add <instructions|skill|prompt|command> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code,cursor,vscode]`
- `agentpack remove <module_id>`

source spec：
- `local:<path>`（repo 内相对路径）
- `git:<url>#ref=<ref>&subdir=<path>`

例子：
- `agentpack add instructions local:modules/instructions/base --id instructions:base --tags base`
- `agentpack add skill git:https://github.com/your-org/agentpack-modules.git#ref=v1.2.0&subdir=skills/git-review --id skill:git-review --tags work`

## lock / fetch / update

- `agentpack lock`：生成/更新 `agentpack.lock.json`
- `agentpack fetch`：按 lockfile 拉取外部 sources 到 cache/store
- `agentpack update`：组合命令
  - 默认：lockfile 不存在时执行 lock+fetch；存在时默认只 fetch
  - flags：`--lock`/`--fetch`/`--no-lock`/`--no-fetch`

## preview / plan / diff

- `agentpack plan`：展示将要发生的 create/update/delete（不写入）
- `agentpack diff`：对当前计划输出 diff
- `agentpack preview [--diff]`：组合命令（总是 plan；加 `--diff` 时同时 diff）

说明：
- 计划中 update 可能是两类：
  - `managed_update`：更新托管文件
  - `adopt_update`：目标路径存在但非托管，默认拒绝覆盖（见 deploy 的 `--adopt`）

## deploy

`agentpack deploy [--apply] [--adopt]`

- 不带 `--apply`：只展示计划与 diff（相当于“plan + diff”）
- 带 `--apply`：写入目标目录、生成 snapshot，并写入每个 target root 的 `.agentpack.manifest.json`
- 若计划包含 `adopt_update`：必须显式给 `--adopt` 才允许覆盖写入（否则报 `E_ADOPT_CONFIRM_REQUIRED`）

常用：
- `agentpack deploy --apply`
- `agentpack --json deploy --apply --yes`
- `agentpack deploy --apply --adopt`

## status

`agentpack status`
- 基于 `.agentpack.manifest.json` 检测 drift（missing/modified/extra）
- 若缺少 manifest（首次使用或旧版本迁移），会降级为“desired vs FS”的对比并给 warning

## rollback

`agentpack rollback --to <snapshot_id>`
- 回滚到某次部署/引导产生的快照

## doctor

`agentpack doctor [--fix]`
- 检查 machineId、目标目录可写性、常见配置错误
- `--fix`：在检测到的 git repo 的 `.gitignore` 中追加 `.agentpack.manifest.json`（避免误提交）

## remote / sync

- `agentpack remote set <url> [--name origin]`：配置 config repo 的 git remote
- `agentpack sync [--rebase] [--remote origin]`：pull/rebase + push 的推荐同步流程

## bootstrap

`agentpack bootstrap [--scope user|project|both]`
- 安装 operator assets：
  - Codex：operator skill
  - Claude Code：`/ap-*` 命令集合

提示：target 选择使用全局 `--target`：
- `agentpack --target codex bootstrap --scope both`

## overlay

- `agentpack overlay edit <module_id> [--scope global|machine|project] [--sparse|--materialize]`
- `agentpack overlay rebase <module_id> [--scope ...] [--sparsify]`（3-way merge；支持 `--dry-run`）
- `agentpack overlay path <module_id> [--scope ...]`

## explain

`agentpack explain plan|diff|status`
- 解释某个变更/漂移来自哪个 module，来自哪一层 overlay（upstream/global/machine/project）

## record / score

- `agentpack record`：从 stdin 读取 JSON，写入 `state/logs/events.jsonl`
- `agentpack score`：基于 events 统计模块成功率/失败率（容忍坏行，输出 warnings）

## evolve

- `agentpack evolve propose [--module-id <id>] [--scope global|machine|project] [--branch <name>]`
  - 捕获 drifted deployed 内容，生成 overlay proposal（创建分支并写文件）
  - 推荐先 `--dry-run --json` 看候选
- `agentpack evolve restore [--module-id <id>]`
  - 恢复 missing 的 desired outputs（create-only；支持 `--dry-run`）

## completions

`agentpack completions <shell>`
- 生成 shell completion 脚本（bash/zsh/fish/powershell 等）
