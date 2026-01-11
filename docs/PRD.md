# PRD.md

## 1. 背景与问题

AI coding 工具生态（Codex CLI、Claude Code、Cursor、VS Code Copilot 等）都在引入可扩展“资产”：
- 指令/规则类：AGENTS.md、各种 rules 文件
- 可复用能力单元：skills（Agent Skills 标准）
- 可复用 prompt：/prompts、/commands
- 子代理/agents、工作流脚本（可选）

现实痛点：
1) 发现与筛选成本高：同一类能力（如 git review）有上百实现，缺少统一“可理解/可验证/可回滚”的体验。
2) 安装与管理碎片化：不同工具存放位置/加载规则不一，导致复制粘贴+手动同步。
3) 更新与回滚困难：上游更新、自己定制、项目间版本不一致，容易漂移。
4) 多台电脑一致性：dotfiles+symlink 的传统做法在部分工具上不可靠（导致必须复制，进而漂移）。
5) 进化/优化缺一套工程闭环：从“感觉差点意思”到“可重复改进”，缺少 overlay + diff + 评估 gate。

## 2. 产品愿景

Agentpack：一个 AI-first 的本地“资产控制平面”，用一份源配置（manifest + overlays + lockfile）管理 prompts / skills / commands / instructions，并一键部署到多个 AI coding 客户端。

核心原则：
- Single source of truth（单一真源）
- 生成式部署（plan -> diff -> apply），默认 copy/render（不依赖 symlink）
- 可复现（lockfile）、可回滚（deploy snapshots）
- AI-first：CLI 是机器可调用 API（--json），并自动生成“operator skills/commands”让 Codex/Claude 能自操作 agentpack

## 3. 目标用户与场景

主要用户（v0.1）：
- 深度 AI coding 用户：同时使用 Codex CLI + Claude Code（以及可能的 Cursor/VS Code），需要跨项目/跨机器稳定。
- 小团队：希望有一套“团队默认资产”+“项目覆盖（overlay）”的机制。

典型场景：
- “我装了 20 个 skills/commands，想按项目分组启用，并在多台电脑一致”
- “某个 skill 不够好，我要基于自己习惯做定制，但还想跟随上游更新”
- “我希望 AI 能自己根据需求创建/更新 assets，并通过 diff+评估 gate 安全落地”

## 4. 目标与非目标

### v0.1 目标（必须）
1) 管理 4 类资产：instructions（AGENTS.md）、skills（Agent Skills）、commands（Claude slash commands）、prompts（Codex custom prompts）
2) Profiles：按场景分组启用资产（default/work/research 等）
3) Overlays：对任意资产做项目级/用户级覆盖（不改 upstream）
4) 一键部署到目标：
   - Codex：~/.codex/skills、~/.codex/prompts、~/.codex/AGENTS.md + repo 内 AGENTS.md、repo 内 .codex/skills（可选）
   - Claude Code：.claude/commands、~/.claude/commands（skills 先做最小支持）
5) plan/diff/apply/rollback：可审计、可回滚
6) AI-first：所有核心命令支持 --json；提供 agentpack-operator（Codex Skill + Claude Commands）自举

### v0.2 目标（重要增强）
1) 多机器一致性：`remote set` + `sync --rebase` 固化推荐的同步路径
2) 部署安全：target manifests（`.agentpack.manifest.json`）+ 删除保护
3) machine overlays：global → machine → project 的覆盖层级 + `--machine`
4) AI-first 可用：`doctor` 自检 + `schema_version` 的稳定 JSON 输出
5) 进化最小闭环：`record`/`score`/`explain`/`evolve propose`（先提案，不自动 apply）

### v0.1 非目标（明确不做）
- 不做 MCP server 的安装/运行/依赖管理（后续做 module type 占位即可）
- 不做云端账户/服务端同步（先 Git 作为同步方式）
- 不做完整 GUI（先 CLI + TUI；GUI 未来可选）
- 不做“全自动自我进化直接落地”（先做 AI 辅助生成 patch + 人工确认 + 可选 eval gate）

## 5. 关键产品决策

1) 默认 copy/render 部署，不默认 symlink
原因：Codex/Claude 在 symlink 资产发现上存在已知不稳定性（参考资料见文末）。

2) Claude 默认文件落盘模式；插件模式作为可选“打包输出”
原因：插件安装会复制到 cache，且对路径引用有约束；插件缓存更新也可能出现 stale 问题。

3) Codex prompts 仅支持 user scope（~/.codex/prompts）
原因：Codex 文档说明 custom prompts 位于本地 Codex home（~/.codex），不通过 repo 共享；如需共享应使用 skills。

## 6. 成功指标（可量化）

- 资产接入时间：从“找到一个 asset”到“在 Codex/Claude 都可用”的中位时间 < 3 分钟
- 回滚时间：任意 deploy 回滚 < 30 秒
- 漂移率：agentpack status 检测到的 drift 次数/周持续下降
- AI self-serve：在 Codex/Claude 中通过 operator assets 完成 agentpack 操作的比例逐步上升（>=30%）

## 7. 风险与应对

- 资产发现规则/目录规则随工具升级变化：用 adapters 隔离；提供 adapter 测试与快速修复机制
- 插件缓存/更新问题：默认不依赖插件；插件模式标记为高级输出
- 安全风险：所有 apply 前展示 diff；对 Claude Bash 工具最小 allowed-tools；默认不执行第三方脚本
- “进化”导致退化：引入 eval gate（v0.2），先从可选脚本 eval 开始

## 8. 里程碑（建议）

M0（1-2 周）：核心数据模型 + manifest/lock/store + codex 部署（skills/prompts/AGENTS）+ plan/diff/apply/rollback
M1（1-2 周）：Claude commands 部署 + operator 自举 + TUI v0
M2（2-4 周）：overlays 完整化（3-way merge 提示）+ basic eval gate + 初步 registry 搜索
M3（可选）：插件输出模式 + Cursor/VS Code adapters + MCP module 占位

## 9. 参考资料（原始链接）
- AGENTS.md: https://agents.md/
- Codex AGENTS.md 指令发现：https://developers.openai.com/codex/guides/agents-md/
- Codex Skills：https://developers.openai.com/codex/skills/
- Codex Create Skill（symlink/校验注意）：https://developers.openai.com/codex/skills/create-skill/
- Agent Skills spec：https://agentskills.io/specification
- Codex Custom Prompts：https://developers.openai.com/codex/custom-prompts/
- Codex prompts symlink issue：https://github.com/openai/codex/issues/4383
- Claude Code Slash Commands：https://code.claude.com/docs/en/slash-commands
- Claude .claude symlink bug：https://github.com/anthropics/claude-code/issues/10522
- Claude plugin marketplaces（cache/路径限制）：https://code.claude.com/docs/en/plugin-marketplaces
- Claude plugin cache stale issues：
  - https://github.com/anthropics/claude-code/issues/15642
  - https://github.com/anthropics/claude-code/issues/14061
