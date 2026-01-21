# Architecture（架构）

> Language: 简体中文 | [English](../../explanation/architecture.md)

## 1. 一句话总结

Agentpack = “声明式资产编译器 + 安全的应用器（applier）”。

它把 **配置（manifest）**、**版本（lockfile）** 与 **本地定制（overlays）** 组合起来，生成各工具可发现的最终文件，并通过 manifest/snapshots 保障可回滚与安全删除。

## 2. 架构图（高层）

```mermaid
flowchart TD
  M[agentpack.yaml<br/>manifest] --> C[Compose & materialize<br/>(per module)]
  L[agentpack.lock.json<br/>lockfile] --> C
  O[overlays<br/>(global / machine / project)] --> C

  C --> R[Render desired state<br/>(per target)]
  R --> P[Plan / Diff]
  P -->|dry run| OUT[Human output / JSON envelope]
  P -->|deploy --apply| A[Apply (writes)]

  A --> MF[Write target manifest<br/>.agentpack.manifest.&lt;target&gt;.json]
  A --> SS[Create snapshot<br/>state/snapshots/]
  A --> EV[Record events<br/>state/logs/]

  SS --> RB[Rollback]
```

## 3. 三层存储模型（刻意分离）

A) Config repo（git 管理、可同步、可审计）
- `agentpack.yaml`
- `modules/`（可选：仓库内模块）
- `overlays/` 与 `projects/`（定制与反馈闭环）

B) Cache/store（不进 git）
- git sources 的 checkout / cache
- 目标：可复现，不强求可审计

C) Deployed outputs（不进 git）
- 写入到目标工具的目录/文件
- 目标：始终可重建；rollback 依赖 snapshots

## 4. 关键目录

默认 `AGENTPACK_HOME=~/.agentpack`（可覆写）：
- `repo/`：config repo
- `cache/`：git sources cache
- `state/snapshots/`：deploy/rollback snapshots
- `state/logs/`：events.jsonl（record/score）

## 5. 核心流程（engine）

1) Load：读 `agentpack.yaml`，并使用 lockfile（若存在）做可复现解析；推导 `project_id` 与 `machine_id`。

2) Materialize：逐 module 合成 upstream → global → machine → project，并验证模块结构（`AGENTS.md` / `SKILL.md` / 单文件 `.md` 等）。

3) Render：按 target 生成最终 desired state（写到目标工具可发现的位置）。

4) Plan / Diff：计算 create/update/delete；对覆盖未受管文件的写入标记为 `adopt_update`，默认拒绝，需显式 adopt。

5) Apply：执行写入（通常走 staging + 原子替换），刷新 per-target manifest，并记录 snapshot 以支持 rollback。

## 6. 安全与自动化护栏

- `--json` 输出是稳定 envelope（`schema_version=1`），失败时也保持合法 JSON。
- `--json` 模式下所有写入类命令必须显式 `--yes`（防止脚本/LLM 误写盘）。
- 稳定错误码见 `docs/ERROR_CODES.md`。
