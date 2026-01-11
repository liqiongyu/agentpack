# README.md

## Agentpack

一个 AI-first 的“资产控制平面（control plane）”，用一份清单（manifest）+ 覆盖层（overlays）+ 锁文件（lockfile），在本地统一管理并部署：
- AGENTS.md / instructions
- Agent Skills（SKILL.md）
- Claude Code slash commands（.claude/commands）
- Codex custom prompts（~/.codex/prompts）

目标：跨项目、跨机器、跨工具保持一致，可更新、可回滚，并且让 AI 自己会用（operator assets）。

## 为什么不是 dotfiles + symlink？

因为现实里：
- Codex skills 会忽略 symlinked directories，prompts symlink 也可能不被发现
- Claude Code 也出现过 `.claude` 是 symlink 时 slash commands 不识别的 bug

所以 agentpack 默认用 copy/render（生成真实文件），以稳定性优先。

参考链接见文末。

## 快速开始（v0.4 期望体验）

1) 初始化 agentpack repo（单一真源）：
  agentpack init

2) （可选）配置远端并同步：
  agentpack remote set https://github.com/you/agentpack-config.git
  agentpack sync --rebase

3) 自检（machineId + 目标目录可写性）：
  agentpack doctor

4) 添加一些模块（示例）：
  agentpack add instructions local:modules/instructions/base --id instructions:base --tags base
  agentpack add skill git:https://github.com/your-org/agentpack-modules.git#ref=v1.0.0&subdir=skills/git-review --id skill:git-review --tags work
  agentpack add prompt local:modules/prompts/draftpr.md --id prompt:draftpr --tags work
  agentpack add command local:modules/claude-commands/ap-plan.md --id command:ap-plan --tags base --targets claude_code

5) 锁版本并拉取依赖（组合命令）：
  agentpack update

6) 预览变更：
  agentpack preview --profile work --diff

7) 部署（会备份、可回滚；并写入 `.agentpack.manifest.json`）：
  agentpack deploy --profile work --apply --yes --json

8) 查看状态与漂移：
  agentpack status --profile work

9) 回滚到某次部署：
  agentpack rollback --to <snapshot_id>

## AI-first：让 Codex / Claude 自己会用 agentpack

执行一次：
  agentpack bootstrap --target all --scope both

它会安装：
- Codex: 一个 agentpack-operator skill（教 Codex 用 agentpack CLI，并优先用 --json）
- Claude Code: /ap-plan /ap-deploy /ap-status /ap-diff 等 slash commands（使用最小 allowed-tools）

你的目标体验是：
- 你在对话里说“把这个项目的 review skill 优化一下并部署”，AI 会先 /ap-plan 看 diff，再 /ap-deploy 应用。

## 配置与目录结构

默认数据目录：`~/.agentpack`（可通过 `AGENTPACK_HOME` 覆盖）

内部三层：
- repo/（Git）：manifest + overlays（可同步）
- cache/：外部拉取内容（不进 Git）
- state/snapshots/：deploy snapshots（不进 Git）
- state/logs/：record events（不进 Git）

## 贡献指南（简要）

欢迎贡献：
- 新 target adapter（Cursor/VS Code）
- modules registry（索引/搜索）
- overlay 3-way merge
- TUI

建议先从 adapters 的 golden tests 开始，保证升级不破坏部署语义。

## 参考资料
- AGENTS.md: https://agents.md/
- Codex AGENTS.md 发现链条: https://developers.openai.com/codex/guides/agents-md/
- Codex Skills（含 scope/优先级）: https://developers.openai.com/codex/skills/
- Codex Create Skill（symlink 限制）: https://developers.openai.com/codex/skills/create-skill/
- Codex Custom Prompts（仅 ~/.codex）: https://developers.openai.com/codex/custom-prompts/
- Codex prompts symlink issue: https://github.com/openai/codex/issues/4383
- Claude Code Slash Commands（allowed-tools + !bash）: https://code.claude.com/docs/en/slash-commands
- Claude .claude symlink bug: https://github.com/anthropics/claude-code/issues/10522
- Claude plugin marketplaces（cache/路径限制）: https://code.claude.com/docs/en/plugin-marketplaces
- Claude plugin cache stale issues:
  - https://github.com/anthropics/claude-code/issues/15642
  - https://github.com/anthropics/claude-code/issues/14061
