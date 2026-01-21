# 对比：Agentpack vs dotfiles managers

> Language: 简体中文 | [English](../../explanation/compare-dotfiles-managers.md)

Agentpack **不是**通用的 dotfiles manager。它更像一个本地“资产控制面”，聚焦把 AI 编程相关资产（`AGENTS.md`、skills、prompts、commands）部署到各工具的**可发现位置**，并提供安全护栏与自动化契约。

不过，dotfiles managers 与 Agentpack 很适合组合使用。

## 快速对比

| 维度 | GNU Stow | chezmoi | yadm | Agentpack |
| --- | --- | --- | --- | --- |
| 主要对象 | `$HOME` 的 symlink farm | 多机 dotfiles 管理 | 把 dotfiles 作为 `$HOME` 下的 git repo | 跨工具部署 coding-agent 资产 |
| 部署模型 | symlink | copy/symlink/template（工具化） | git +（可选）模板/钩子 | 按 **target** 渲染 desired state + apply |
| 安全与回滚 | 依赖 git（自行约束） | git 回滚 + 工具能力 | git 回滚 + 工具能力 | snapshots + rollback + manifest 安全删除 |
| targets | 不适用（自定义布局） | 不适用（多以 `$HOME` 为中心） | 不适用（多以 `$HOME` 为中心） | 内置 target adapters（Codex/Claude Code/…） |
| overlays / rebase | 不适用 | 模板/机器差异 | 模板/条件化 | overlays 分层 + `overlay rebase`（3-way merge） |
| 自动化契约 | shell 脚本 | shell 脚本 | shell 脚本 | 稳定 `--json` envelope + MCP tools（写入需确认） |

## 什么时候它们更合适（选 dotfiles manager）

- 你要管理的是 **全部** `$HOME` dotfiles（shell/editor/git/ssh 等），并希望统一治理。
- 你强依赖 **模板** 与机器差异（条件化渲染）来适配大量配置。
- 你的核心诉求是 **symlink/家目录整洁**，并能接受对应的风险与维护方式。

## 什么时候 Agentpack 更合适

- 你希望把多工具（Codex/Claude Code/Cursor/VSCode/…）的 agent 资产统一到一个**单一真相源**。
- 你需要显式的 **targets**（映射规则/校验/一致性测试），而不是“把文件丢进 `$HOME` 就完了”。
- 你需要更安全的自动化：**先 preview/diff**、显式 adopt/confirm、稳定机器可读输出（`--json` / MCP）。

## 什么时候不该用 Agentpack

- 你的目标是治理整个 `$HOME` 的 dotfiles（shell/editor/git/ssh），而不需要工具侧的 target 映射与校验。
- 你强依赖复杂模板与条件化渲染来管理大量配置（通常 dotfiles manager 更匹配）。
- 你更偏好 symlink-first 且尽量少工具依赖；Agentpack 的 snapshot/manifest 语义可能是过度设计。

## 推荐组合方式

- 用 chezmoi / yadm / Stow 管理你的 dotfiles。
- 用 Agentpack 只管理 agent 资产：targets/overlays/rollback 语义能发挥更大价值。

## 参考（官方文档）

- GNU Stow manual: https://www.gnu.org/s/stow/manual/stow.html
- chezmoi: https://chezmoi.io/
- yadm: https://yadm.io/
