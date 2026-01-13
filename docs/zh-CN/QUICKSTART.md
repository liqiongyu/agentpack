# 快速开始（v0.5.0）

> Language: 简体中文 | [English](../QUICKSTART.md)

本指南的目标：从 0 到第一次成功部署（deploy），并理解最核心的安全边界（不误删、不误覆盖）。

## 0) 安装

- Rust 用户：
  - `cargo install agentpack --locked`
- 非 Rust 用户：
  - 从 GitHub Releases 下载对应平台二进制（见仓库 README）。

验证：
- `agentpack help`

## 1) 初始化你的配置仓库（config repo）

Agentpack 默认在 `~/.agentpack/repo` 创建/读取配置仓库（可用 `AGENTPACK_HOME` 或 `--repo` 改位置）。

1. 初始化 skeleton：
- `agentpack init`

会生成：
- `agentpack.yaml`（清单）
- `modules/`（示例模块目录：instructions/prompts/claude-commands）

建议你把它变成一个真正的 git repo（agentpack **不会**自动 `git init`）：
- `cd ~/.agentpack/repo && git init && git add . && git commit -m "init agentpack"`

可选：配置远端并同步（多机器）：
- `agentpack remote set <your_git_url>`
- `agentpack sync --rebase`

## 2) 配置 targets（Codex / Claude Code）

`agentpack init` 会写入一份可用的 targets 默认配置。你通常只需要按需调整 options。

常见最小建议：
- Codex：写 user skills、repo skills、global/repo AGENTS.md、user prompts（prompts 只支持 user scope）
- Claude Code：写 repo commands + user commands（skills 默认先关闭，按需开启）

先跑一次自检：
- `agentpack doctor`

如果你看到权限/目录不存在的告警，按提示创建目录或修正 `agentpack.yaml` 里的路径选项。

## 3) 添加模块（modules）

模块写在 `agentpack.yaml -> modules:` 里。你可以用命令添加（推荐，少踩坑）：

- 添加一份 instructions（目录模块）：
  - `agentpack add instructions local:modules/instructions/base --id instructions:base --tags base`

- 添加一个 Codex prompt（单文件模块）：
  - `agentpack add prompt local:modules/prompts/draftpr.md --id prompt:draftpr --tags base`

- 添加一个 Claude slash command（单文件模块）：
  - `agentpack add command local:modules/claude-commands/ap-plan.md --id command:ap-plan --tags base --targets claude_code`

你也可以添加 git 模块（锁到 commit，可复现）：
- `agentpack add skill git:https://github.com/your-org/agentpack-modules.git#ref=v1.2.0&subdir=skills/git-review --id skill:git-review --tags work`

## 4) 锁版本并拉取依赖（update）

推荐用组合命令：
- `agentpack update`

行为：
- 若 `agentpack.lock.json` 不存在：会先 lock 再 fetch
- 若已存在：默认只 fetch

## 5) 预览（preview）

先看会发生什么：
- `agentpack preview --diff`

常见你会看到：
- 将要写入哪些 target
- 哪些文件 create/update/delete
- diff（如果加了 `--diff`）

提示：如果你只想看某个 profile 或 target：
- `agentpack --profile work preview --diff`
- `agentpack --target codex preview --diff`

## 6) 部署（deploy）

执行写入需要 `--apply`：
- `agentpack deploy --apply`

安全边界：
- 删除只会删除“托管文件”（见每个 target root 的 `.agentpack.manifest.json`），不会删用户非托管文件。
- 覆盖保护：如果目标路径存在但**不属于托管文件**，会被标记为 `adopt_update`，默认拒绝覆盖。

如果你确认要“接管并覆盖”这类文件：
- `agentpack deploy --apply --adopt`

如果你在自动化里用 `--json`：
- 所有写入类命令都需要显式 `--yes`
  - `agentpack --json deploy --apply --yes`
  - 若存在 adopt_update，还需要 `--adopt`

## 7) 漂移检查（status）与回滚（rollback）

- 查看漂移：
  - `agentpack status`

- 回滚到某次部署快照：
  - `agentpack rollback --to <snapshot_id>`

快照 id 会在 `deploy --apply` 成功后输出（JSON 里在 `data.snapshot_id`）。

## 8) 改动沉淀：overlays

当你想基于上游模块做本地改动（并希望未来还能合并上游更新），用 overlays：

- 创建并编辑 overlay（默认 global scope）：
  - `agentpack overlay edit <module_id>`

- 更推荐的做法：创建稀疏 overlay（不复制整棵上游树，只放改动文件）：
  - `agentpack overlay edit <module_id> --sparse`

- 需要浏览上游文件时再补齐（missing-only，不覆盖已改文件）：
  - `agentpack overlay edit <module_id> --materialize`

- 上游更新后，把 overlay 3-way merge 到最新上游：
  - `agentpack overlay rebase <module_id> --sparsify`

## 9) AI-first 自举（bootstrap）

让 AI 自己会用 agentpack（推荐完成一次）：
- `agentpack bootstrap --scope both`

它会安装：
- Codex：一个 operator skill（教 Codex 调 agentpack CLI，优先用 `--json`）
- Claude Code：一组 `/ap-*` slash commands（计划/部署/状态/提案等）

更多见：`BOOTSTRAP.md`。
