# PRD.md

> Current as of **v0.5.0** (2026-01-13). Historical content is tracked in git history.

## 1. 背景

你会同时使用多个 AI coding 工具（例如 Codex CLI、Claude Code 等），而这些工具各自支持一套可插拔资产：AGENTS.md、skills、slash commands、prompts……

现实痛点通常来自：
- 发现成本：同类能力有大量实现，需要筛选与对比。
- 维护成本：安装后会不断微调；上游更新后需要合并。
- 协同成本：多台电脑、多项目、多工具导致版本与路径不一致。
- 可控性：希望可审计、可回滚、可复现，而不是“当前生效的是哪个版本我不确定”。

Agentpack 的定位：在本地提供一个“资产控制平面（control plane）”，用声明式清单 + overlays + lockfile，把资产从管理到分发再到回滚统一起来，并对 AI 与人类都友好（AI-first）。

## 2. 产品目标

P0（必须）
- **可复现**：git sources 通过 lockfile 锁到 commit + sha256。
- **可回滚**：deploy/rollback snapshots。
- **安全写入**：仅删除托管文件；覆盖非托管文件必须显式 adopt。
- **可编排**：`--json` 输出稳定；写入类命令在 `--json` 下必须 `--yes`。

P1（强烈建议）
- overlays 的编辑/合并体验足够顺滑（sparse overlay + rebase）。
- 能把 drift 回流成可 review 的改动（evolve propose）。

## 3. v5 milestone 已实现能力（当前实现）

闭环能力：
- Config repo（manifest + overlays）作为单一真源
- lockfile（`agentpack.lock.json`）+ store/cache（git checkout）
- plan/diff/deploy：计划、diff、带备份写入与 snapshots
- 每个 root 写 `.agentpack.manifest.json`：安全删除、可靠 drift/status
- 覆盖保护：`adopt_update` 需要 `--adopt`

体验与 AI-first：
- 组合命令：`update`（lock+fetch）/ `preview`（plan + 可选 diff）
- overlays 三 scope：global/machine/project
- sparse overlay + `overlay rebase`（3-way merge）
- `doctor --fix`：降低 manifest 被误提交风险
- bootstrap：安装 operator assets（Codex operator skill + Claude `/ap-*` commands）
- evolve：`propose`（把 drift 变成 overlays 提案分支）+ `restore`（create-only 恢复 missing）

## 4. 非目标（短期）

- Registry/marketplace 的完整发现体系（先以 git/local 为主）
- 复杂 patch-based overlays（更强三方合并/patch 模型）
- GUI（先保持 CLI；需要时再做轻量 TUI）

## 5. 参考（上游官方文档）

- AGENTS.md: https://agents.md/
- Codex:
  - https://developers.openai.com/codex/guides/agents-md/
  - https://developers.openai.com/codex/skills/
  - https://developers.openai.com/codex/custom-prompts/
- Claude Code:
  - https://code.claude.com/docs/en/slash-commands
