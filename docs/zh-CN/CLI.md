# CLI 参考

> Language: 简体中文 | [English](../CLI.md)

本文件面向“想快速查命令怎么用”的场景。更偏工作流的内容见：`WORKFLOWS.md`。

## 全局参数（所有命令都支持）

- `--repo <path>`：指定 config repo 路径（默认 `$AGENTPACK_HOME/repo`）
- `--profile <name>`：选择 profile（默认 `default`）
- `--target <codex|claude_code|cursor|vscode|jetbrains|zed|all>`：选择 target（默认 `all`）
- `--machine <id>`：覆盖 machineId（用于 machine overlays；默认自动探测）
- `--json`：stdout 输出机器可读 JSON（envelope）
- `--yes`：跳过确认（注意：`--json` 下写入类命令必须显式给）
- `--dry-run`：强制 dry-run（即使传了 `deploy --apply` 或 `overlay rebase` 等也不写入）

提示：
- `agentpack help --json` 会返回结构化的命令列表与 mutating 命令集合。
- `agentpack schema --json` 会返回 JSON envelope 与常用命令的 data 字段提示。

## init

`agentpack init [--git] [--bootstrap] [--guided]`
- 初始化 config repo skeleton（创建 `agentpack.yaml` 与示例目录）
- 默认不会自动 `git init`
- `--guided`：交互式引导（需要真实 TTY）来生成最小可用的 `agentpack.yaml`（targets、scope、是否 bootstrap）
- `--git`：同时初始化 git repo，并确保 `.gitignore` 忽略 `.agentpack.manifest*.json`
- `--bootstrap`：同时安装 operator assets 到 config repo（等价于执行 `agentpack bootstrap --scope project`）

说明：
- `init --guided` 要求 stdin/stdout 都是终端。在 `--json` 模式下若不是 TTY，会返回 `E_TTY_REQUIRED`。

## add / remove

- `agentpack add <instructions|skill|prompt|command> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code,cursor,vscode,jetbrains,zed]`
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
- 带 `--apply`：写入目标目录、生成 snapshot，并写入每个 target root 的 `.agentpack.manifest.<target>.json`
- 若计划包含 `adopt_update`：必须显式给 `--adopt` 才允许覆盖写入（否则报 `E_ADOPT_CONFIRM_REQUIRED`）

常用：
- `agentpack deploy --apply`
- `agentpack --json deploy --apply --yes`
- `agentpack deploy --apply --adopt`

## status

`agentpack status [--only <missing|modified|extra>[,...]]`
- 基于 `.agentpack.manifest.<target>.json` 检测 drift（missing/modified/extra）（出于兼容性也会读取 legacy manifests）
- 若缺少 manifest（首次使用或旧版本迁移），会降级为“desired vs FS”的对比并给 warning
- `--only`：只展示指定 kind 的 drift（可重复传参或用逗号分隔）

## tui（可选）

`agentpack tui [--adopt]`

- Feature-gated：需用 `--features tui` 编译启用该命令。
- 不支持 `--json` 输出：若传了 `--json` 会报 `E_CONFIG_INVALID`。
- 交互式 TUI，用于浏览 `plan` / `diff` / `status`。
- `a`：触发 apply，并弹出明确的确认提示（对当前 `--profile` / `--target` 执行等价于 `deploy --apply` 的写入）。
- `--adopt`：允许覆盖非托管文件（adopt updates），语义与 `deploy --adopt` 一致。

按键说明见 `TUI.md`。

## rollback

`agentpack rollback --to <snapshot_id>`
- 回滚到某次部署/引导产生的快照

## doctor

`agentpack doctor [--fix]`
- 检查 machineId、目标目录可写性、常见配置错误
- `--fix`：在检测到的 git repo 的 `.gitignore` 中追加 `.agentpack.manifest*.json`（避免误提交）

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

- `agentpack overlay edit <module_id> [--scope global|machine|project] [--kind dir|patch] [--sparse|--materialize]`
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

## policy（治理，opt-in）

Governance 命令位于 `policy` 命名空间，面向 CI 友好。核心命令不会读取 `repo/agentpack.org.yaml`。

- `agentpack policy lint`：lint operator assets 与组织 policy 约束（只读）
- `agentpack policy audit`：从 `repo/agentpack.lock.json` 生成审计报告（只读）
  - 在 git 历史可用时，尽力提供 lockfile 变更摘要
  - 当存在 `repo/agentpack.org.lock.json` 时，包含 policy pack pin 详情
- `agentpack policy lock`：解析并 pin 配置的 policy pack（写入 `repo/agentpack.org.lock.json`）

## completions

`agentpack completions <shell>`
- 生成 shell completion 脚本（bash/zsh/fish/powershell 等）
