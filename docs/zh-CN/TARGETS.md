# Targets（codex / claude_code / cursor / vscode / jetbrains）

> Language: 简体中文 | [English](../TARGETS.md)

Target 决定 agentpack 要把“编译后的资产”写到哪里，以及哪些目录需要写 `.agentpack.manifest.json` 来实现安全删除和漂移检测。

目前内置 targets：
- `codex`
- `claude_code`
- `cursor`
- `vscode`
- `jetbrains`

Target 的通用字段见 `CONFIG.md`。

## 1) codex

### 写入位置（roots）

由 `targets.codex.scope` 和 `targets.codex.options.*` 决定，可能包含：
- `~/.codex`（global instructions：`AGENTS.md`）
- `~/.codex/skills`（user skills）
- `~/.codex/prompts`（custom prompts；只支持 user scope）
- `<project_root>/AGENTS.md`（project instructions）
- `<project_root>/.codex/skills`（repo skills）

说明：
- `project_root` 来自当前工作目录的 project 识别（通常是 git repo root）。
- 每个 root 都会写入（或更新）`.agentpack.manifest.json`，用于安全删除与 drift。

### module → 输出映射

- `instructions`
  - 收集每个 instructions module 的 `AGENTS.md` 内容
  - 多个模块时会合成一个 `AGENTS.md`：用 per-module section markers 标记来源，以支持 `evolve propose` 对聚合文件回溯

- `skill`
  - 复制 module 目录下所有文件到：
    - `~/.codex/skills/<skill_name>/...`（如启用 user skills）
    - `<project_root>/.codex/skills/<skill_name>/...`（如启用 repo skills）
  - `<skill_name>` 默认从 module id 推导（`skill:<name>`），否则会做一次安全规整

- `prompt`
  - 复制单个 `.md` 文件到 `~/.codex/prompts/<filename>.md`

### 常用 options

- `codex_home`：默认 `"~/.codex"`
- `write_repo_skills`：默认 true（需要 project scope 允许）
- `write_user_skills`：默认 true（需要 user scope 允许）
- `write_user_prompts`：默认 true（需要 user scope 允许）
- `write_agents_global`：默认 true（需要 user scope 允许）
- `write_agents_repo_root`：默认 true（需要 project scope 允许）

### 限制与建议

- agentpack 默认使用 copy/render，不依赖 symlink（目标是让 Codex 稳定发现）。
- prompts 按 Codex 语义只写 user scope（`~/.codex/prompts`）。如果你想共享“可复用能力”，更推荐写成 skill。

## 2) claude_code

### 写入位置（roots）

- `~/.claude/commands`（user commands；默认启用）
- `<project_root>/.claude/commands`（repo commands；默认启用）
- `~/.claude/skills`（user skills；默认关闭）
- `<project_root>/.claude/skills`（repo skills；默认关闭）

### module → 输出映射

- `command`
  - 复制单个 `.md` 文件到 commands 目录
  - 文件名就是 slash command 名（例如 `ap-plan.md` → `/ap-plan`）

- `skill`
  - 复制 module 目录下所有文件到：
    - `~/.claude/skills/<skill_name>/...`（如启用 user skills）
    - `<project_root>/.claude/skills/<skill_name>/...`（如启用 repo skills）
  - `<skill_name>` 默认从 module id 推导（`skill:<name>`），否则会做一次安全规整

### 常用 options

- `write_repo_commands`：默认 true（需要 project scope 允许）
- `write_user_commands`：默认 true（需要 user scope 允许）
- `write_repo_skills`：默认 false（需要 project scope 允许）
- `write_user_skills`：默认 false（需要 user scope 允许）

### frontmatter 约束（很重要）

Claude Code 的自定义命令文件需要 YAML frontmatter。

最小示例：

```md
---
description: Plan changes with agentpack
allowed-tools:
  - Bash(agentpack*)
  - Bash(git status)
---

# /ap-plan
...
```

规则：
- 必须有 `description`
- 如果正文包含 `!bash` 或 `!\`bash\``：必须声明 `allowed-tools` 且允许 `Bash(...)`

## 3) cursor

Cursor 的 rules 存在 `.cursor/rules`，文件格式为 `.mdc` + YAML frontmatter。

### 写入位置（roots）

- `<project_root>/.cursor/rules`（目前只支持 project scope）

### module → 输出映射

- `instructions`
  - 每个 module 写一个 rule 文件：
    - `<project_root>/.cursor/rules/<module_fs_key>.mdc`
  - 默认 frontmatter：
    - `description: "agentpack: <module_id>"`
    - `globs: []`
    - `alwaysApply: true`

### 常用 options

- `write_rules`：默认 true（需要 project scope）

说明：
- `cursor` 目前只支持 project scope（`scope: user` 会被视为配置错误）。

## 4) vscode

VS Code / GitHub Copilot 使用 repo 级别的 “custom instructions” 和 “prompt files”，默认约定放在 `.github/` 下。

### 写入位置（roots）

- `<project_root>/.github`（instructions；`scan_extras=false`，避免把无关的 `.github/*` 误报为 extra）
- `<project_root>/.github/prompts`（prompt files；`scan_extras=true`）

### module → 输出映射

- `instructions`
  - 合并每个 instructions module 的 `AGENTS.md` 内容到：
    - `<project_root>/.github/copilot-instructions.md`
  - 多个模块时会生成一个带 per-module section markers 的单文件，保留归因信息。

- `prompt`
  - 复制单个 `.md` 文件到：
    - `<project_root>/.github/prompts/<name>.prompt.md`
  - 如果源文件名不以 `.prompt.md` 结尾，agentpack 会自动追加 `.prompt.md` 以便 VS Code 发现。

### 常用 options

- `write_instructions`：默认 true（需要 project scope）
- `write_prompts`：默认 true（需要 project scope）

说明：
- `vscode` 目前只支持 project scope（`scope: user` 会被视为配置错误）。

## 5) jetbrains

JetBrains Junie 默认会从 `.junie/guidelines.md` 加载 project guidelines（也支持像 `AGENTS.md` 这样的 open format）。

这个 target 会把 instructions 写到 Junie 默认路径，让 JetBrains 用户不需要额外 IDE 配置就能生效。

### 写入位置（roots）

- `<project_root>/.junie`（目前只支持 project scope；`scan_extras=true`）

### module → 输出映射

- `instructions`
  - 合并每个 instructions module 的 `AGENTS.md` 内容到：
    - `<project_root>/.junie/guidelines.md`
  - 多个模块时会生成一个带 per-module section markers 的单文件，保留归因信息。

### 常用 options

- `write_guidelines`：默认 true（需要 project scope）

说明：
- `jetbrains` 目前只支持 project scope（`scope: user` 会被视为配置错误）。
- 如果你在 JetBrains 里也用 GitHub Copilot，`vscode` target 生成的 `.github/copilot-instructions.md` 可能也有用（取决于你的 client 是否支持）。

## 6) scan_extras（extra 文件的处理）

某些 roots 会启用 `scan_extras`：
- `true`：status 会报告“目录中存在但不在托管清单里”的 extra 文件（不会自动删除）
- `false`：不扫描 extra（例如 global `~/.codex` 根目录通常不做全量扫描）

## 7) 想加新 target？

看：
- `TARGET_MAPPING_TEMPLATE.md`
- `TARGET_SDK.md`
- `TARGET_CONFORMANCE.md`

## 8) Zed（兼容性）

Agentpack 目前还没有内置 `zed` target。但 Zed 可以从 repo 内的规则文件（例如 `AGENTS.md`、`.github/copilot-instructions.md`）读取项目规则（见：https://zed.dev/docs/context/rules）。

推荐方式：
- 优先使用 `vscode` target 的 instructions 输出（`.github/copilot-instructions.md`），让 Zed 直接读取它。
- 或者使用 `codex` target 的 project instructions 输出（`<project_root>/AGENTS.md`）。

最小示例：

```yaml
targets:
  vscode:
    mode: files
    scope: project
    options:
      write_instructions: true
```
