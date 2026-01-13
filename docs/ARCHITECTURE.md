# ARCHITECTURE.md

> Current as of **v0.5.0** (2026-01-13). Historical content is tracked in git history.

## 1. 一句话总览

Agentpack = “声明式资产编译器 + 安全应用器（apply）”。

输入：
- manifest（想要什么：modules/profiles/targets）
- overlays（怎么改：global/machine/project 三层覆盖）
- lockfile（用哪个版本：git sources 锁到 commit + sha256）

输出：
- 各 target 的可发现目录/文件（例如 `~/.codex/skills/...`、`~/.claude/commands/...`）
- 每个 root 写 `.agentpack.manifest.json`（安全删除与 drift/status）
- state snapshots（deploy/bootstrap/rollback 的快照）

## 2. 三层存储模型（必须分层）

A) Config Repo（建议 git 管理、可同步到远端）
- `agentpack.yaml`
- `modules/`（可选：自研/内置模块）
- `overlays/` 与 `projects/`（定制与回流的改动）

B) Cache/Store（不进 git）
- git sources 的 checkout 缓存
- 目标：可复现，不要求可审计

C) Deployed Outputs（不进 git）
- 写到目标工具目录的最终产物
- 目标：随时可重建；回滚靠 snapshots

## 3. 关键目录

默认 `AGENTPACK_HOME=~/.agentpack`（可覆盖）：
- `repo/`：config repo
- `cache/`：git sources cache
- `state/snapshots/`：部署/回滚快照
- `state/logs/`：events.jsonl（record/score）

## 4. 核心管线（Engine）

1) Load
- 读取 `agentpack.yaml`
- 读取/使用 lockfile（若存在）以获得可复现的 git source 解析
- 解析 project identity（用于 project overlays）与 machine id（用于 machine overlays）

2) Materialize（每个 module）
- resolve upstream module root（local_path 或 store 中的 git checkout）
- 按优先级合成：upstream → global → machine → project
- 校验模块结构：
  - instructions 必有 `AGENTS.md`
  - skill 必有 `SKILL.md`
  - prompt/command 必须只有一个 `.md` 文件
  - command 若使用 bash，frontmatter 必须允许 `Bash(...)`

3) Render（按 target）
- codex：渲染 skills/prompts/AGENTS.md
  - 多个 instructions 合并为一个 `AGENTS.md`，并为每段写 marker，方便后续 `evolve propose` 回溯
- claude_code：渲染 commands（`~/.claude/commands/*.md` 或 `<repo>/.claude/commands/*.md`）

4) Plan / Diff
- 计算 create/update/delete
- 识别 update_kind：
  - managed_update：更新托管文件
  - adopt_update：将覆盖非托管但已存在文件（默认拒绝，需要 `--adopt`）

5) Apply
- 写入前做备份
- 写入后刷新 target manifests（`.agentpack.manifest.json`）
- 记录 snapshot（用于 rollback）

## 5. Overlays（覆盖）

- 目录命名使用 `module_fs_key = sanitize(prefix) + "--" + hash10`（避免 Windows 非法字符与超长路径）。
- 每个 overlay 目录都有 `.agentpack/` 元数据：
  - `baseline.json`：上游指纹（用于 drift warnings 与 3-way merge）
  - `module_id`：原始 module id
- `overlay rebase` 使用 baseline 做 3-way merge，并可 `--sparsify` 删除与上游一致的文件。

## 6. JSON 输出与安全边界

- `--json` 输出是稳定 envelope（schema_version=1），失败时也返回合法 JSON。
- 为防止脚本/LLM误写：`--json` 下写入类命令必须显式 `--yes`。
- 稳定错误码见 `ERROR_CODES.md`。

## 7. 扩展：Target SDK

目标：让新增 target 可预测、可 review、可测试。

- 代码层：`TargetAdapter` trait（`render(...)`）
- 文档层：写清 roots、映射规则、校验规则
- 测试层：conformance（删除保护、manifest、drift、rollback、JSON 契约）

见：`TARGET_SDK.md` 与 `TARGET_CONFORMANCE.md`。
