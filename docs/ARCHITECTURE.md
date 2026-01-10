# ARCHITECTURE.md

## 1. 总览

Agentpack = CLI + Core Engine + Adapters +（可选）TUI

它的本质是“声明式资产编译器”：
- 输入：manifest（想要什么） + overlays（怎么改） + lockfile（锁版本）
- 输出：写入各工具“可发现”的目录/配置文件（生成产物）
- 核心闭环：plan -> diff -> apply -> validate -> snapshot -> rollback

默认策略：copy/render（不依赖 symlink）

## 2. 三层存储模型（必须分层）

A) Config Repo（Git 管理，建议同步到远端）
- 人类需要审计/回滚的内容：
  - agentpack.yaml（清单）
  - overlays（定制）
  - 内置/自研 modules（如果你愿意直接放在 repo 里）
- 目标：可 review、可 PR、可回滚

B) Store/Cache（不进 Git）
- 外部拉取的第三方资产（git checkout / http download）
- 生成中间产物
- 目标：可复现，不要求可审计

C) Deployed Outputs（不进 Git）
- 写入到目标工具目录的“生效形态”
- 目标：随时可重建；回滚靠重新部署 + snapshot

## 3. 目录布局（建议，跨平台用 XDG/OS 标准目录映射）

AGENTPACK_HOME（默认）：
- macOS: ~/Library/Application Support/agentpack
- Linux: ~/.local/share/agentpack
- Windows: %LOCALAPPDATA%\agentpack

结构：
- repo/                 # git config repo（可 push/pull）
  - agentpack.yaml
  - modules/            # 可选：内置或自研 modules
  - overlays/           # 全局 overlays
  - projects/           # 项目级 overlays（按 project_id 分组）
- store/                # cache（gitignored）
- state/
  - deployments/        # 每次 apply 的快照记录
  - machine.json        # 本机信息（不进 Git）
- logs/

## 4. 核心组件

1) CLI Frontend
- 解析 args
- 输出 human-readable 或 JSON（--json）
- 默认不做交互式提问（除非 TUI/--interactive）

2) Config Loader
- 读取 agentpack.yaml
- 在 repo 内解析 profiles/modules/targets
- 读取项目上下文（CWD，git root，git remote）

3) Resolver + Lock
- 将 source（git/local/registry）解析成“具体版本”
- 写入 agentpack.lock.json（版本、commit、sha256、文件列表）
- 提供可复现安装

4) Store Manager
- 拉取/缓存 source 内容
- 校验 sha256
- 暴露“只读”路径给 renderer（避免手改）

5) Overlay Engine
- 支持 3 层覆盖（优先级从低到高）：
  1) upstream module
  2) global overlay（repo/overlays/...）
  3) project overlay（repo/projects/<project_id>/overlays/...）
- 合并策略（v0.1）：
  - 同路径文件：高优先级直接覆盖
  - 目录：递归合并
  - 冲突提示：当 upstream 更新导致 overlay 覆盖的文件已变动时，给出警告并建议人工 review

6) Renderer/Compiler
- 将合成后的 module 渲染成 target 需要的“最终目录结构”
- 支持模板变量（如 {{project.name}}、{{git.remote}}、{{os}}）
- 输出到 temp staging，再交给 apply

7) Adapters（每个 target 一个 adapter）
- detect(): 找到目标路径/规则/权限
- plan(): 计算写入/删除/更新列表（生成 Plan）
- diff(): 输出变更摘要与逐文件 diff
- apply(): 带备份写入
- validate(): 读回验证（文件存在、基本语法校验）
- rollback(snapshot_id): 恢复备份

8) State / Snapshot
- 每次 apply 生成 deployment snapshot：
  - timestamp
  - targets
  - 写入的文件列表
  - 备份位置
  - lockfile hash
- rollback 依赖 snapshot 恢复

## 5. 目标适配策略（v0.1）

### 5.1 Codex Adapter（重点）
- Skills：
  - 支持 locations 与优先级（repo 与 user scope）
  - 默认写入：$REPO_ROOT/.codex/skills/<skill>/SKILL.md（项目）+ ~/.codex/skills（用户，可选）
  - 注意：Codex 忽略 symlinked directories，因此 deploy 必须用真实目录（copy）而非 symlink

- Prompts（Custom Prompts）：
  - 仅 user scope：~/.codex/prompts/*.md
  - 不尝试写 repo scope（文档说明不通过 repo 共享）

- Instructions（AGENTS.md）：
  - 支持写入：
    - ~/.codex/AGENTS.md（全局默认）
    - <repo>/AGENTS.md（项目）
    - （可选）<repo>/<path>/AGENTS.override.md（子目录 override）
  - 遵循 Codex 的发现链条与合并顺序（root -> cwd）

### 5.2 Claude Code Adapter（v0.1 只做文件落盘）
- Slash commands：
  - 写入：<repo>/.claude/commands/*.md（项目）
  - 或写入：~/.claude/commands/*.md（用户）
  - 生成的命令需要 frontmatter：
    - description
    - allowed-tools（如果命令内使用 !`bash`）

- Skills（最小支持）：
  - v0.1 可先占位：repo/.claude/skills 或 ~/.claude/skills（后续再细化）
  - v0.2+ 再做插件输出模式（.claude-plugin/plugin.json + marketplace.json）

## 6. AI-first 设计

### 6.1 CLI 作为机器接口
- 所有核心命令支持 --json
- JSON schema 保持向后兼容（字段新增不破坏）
- 支持 --no-color --quiet --max-bytes 防止 AI “读爆输出”
- 幂等：deploy 重复执行不会产生漂移

### 6.2 自举（operator assets）
- agentpack 内置一个“operator module”，部署后会生成：
  - Codex Skill: agentpack-operator（教 Codex 如何调用 agentpack CLI，如何读 plan/status JSON）
  - Claude slash commands: /ap-plan /ap-deploy /ap-status /ap-diff（用 allowed-tools 限制 bash 调用）

目标：让 AI 在需要时能“自己操作 agentpack”，而不是人类手工。

## 7. 安全策略（v0.1）

- 默认不执行第三方 scripts；只管理文件资产
- apply 前强制展示 diff（除非 --yes）
- 备份必须默认开启
- Claude command 的 allowed-tools 最小化（只允许 agentpack 子命令 + 必要的 git 只读命令）

## 8. 技术栈建议（可选）

- 语言：Rust（便于单文件分发、跨平台、性能；与 Codex CLI 同生态）
- CLI：clap
- 配置：serde_yaml + serde_json
- diff：similar 或 git2 diff
- TUI：ratatui（v0.2）
- git：libgit2/git2 或直接 shell git（但要注意跨平台与安全）

## 9. 参考资料
（同 PRD.md 的链接列表）
