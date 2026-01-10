# SPEC.md

## 0. 约定

命令名：agentpack
配置 repo：agentpack config repo（本地 clone），默认位于 $AGENTPACK_HOME/repo

v0.1 只要求支持：
- target: codex, claude_code
- module types: instructions, skill, prompt, command
- source types: local_path, git (url+ref+subdir)

所有命令默认 human 输出；加 --json 输出机器可读 JSON。

## 1. 核心概念与数据模型

### 1.1 Module
字段（逻辑模型）：
- id: string（全局唯一，建议 type/name）
- type: oneof [instructions, skill, prompt, command]
- source: Source
- enabled: bool（默认 true）
- tags: [string]（用于 profiles）
- targets: [string]（限制仅部署到某些 target；默认 all）
- metadata:
  - name/description（可选）

### 1.2 Source
- local_path:
  - path: string（repo 内相对路径或绝对路径）
- git:
  - url: string
  - ref: string（tag/branch/commit，默认 main）
  - subdir: string（repo 内路径，可为空）
  - shallow: bool（默认 true）

### 1.3 Profile
- name: string
- include_tags: [string]
- include_modules: [module_id]
- exclude_modules: [module_id]

### 1.4 Target
- name: oneof [codex, claude_code]
- mode: oneof [files]（v0.1）
- scope: oneof [user, project, both]
- options: map（target-specific）

### 1.5 Project Identity（用于 project overlays）
project_id 生成规则（优先级）：
1) git remote "origin" URL 标准化后 hash（推荐）
2) 若无 remote：repo root absolute path hash
project_id 必须稳定（同项目多机一致）。

## 2. 配置文件

### 2.1 repo/agentpack.yaml（manifest）

示例：

version: 1

profiles:
  default:
    include_tags: ["base"]
  work:
    include_tags: ["base", "work"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"           # 可覆盖 CODEX_HOME
      write_repo_skills: true          # 写入 $REPO_ROOT/.codex/skills
      write_user_skills: true          # 写入 ~/.codex/skills
      write_user_prompts: true         # 写入 ~/.codex/prompts
      write_agents_global: true        # 写入 ~/.codex/AGENTS.md
      write_agents_repo_root: true     # 写入 <repo>/AGENTS.md
  claude_code:
    mode: files
    scope: both
    options:
      write_repo_commands: true        # 写入 <repo>/.claude/commands
      write_user_commands: true        # 写入 ~/.claude/commands
      write_repo_skills: false         # v0.1 可先关
      write_user_skills: false

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"

  - id: skill:git-review
    type: skill
    tags: ["work"]
    source:
      git:
        url: "https://github.com/your-org/agentpack-modules.git"
        ref: "v1.2.0"
        subdir: "skills/git-review"

  - id: prompt:draftpr
    type: prompt
    tags: ["work"]
    source:
      local_path:
        path: "modules/prompts/draftpr.md"

  - id: command:ap-plan
    type: command
    tags: ["base"]
    source:
      local_path:
        path: "modules/claude-commands/ap-plan.md"

备注：
- instructions module 的 source 指向一个目录，里面可能包含：
  - AGENTS.md（模板）
  - rules fragments（后续扩展）
- skill module 的 source 指向 skill 目录根（包含 SKILL.md）
- prompt module 的 source 指向单个 .md（Codex custom prompt）
- command module 的 source 指向 Claude slash command .md

### 2.2 repo/agentpack.lock.json（lockfile）

最小字段：
- version: 1
- generated_at: ISO8601
- modules: [
  {
    id, type,
    resolved_source: { ... },
    resolved_version: string (commit sha or semver tag),
    sha256: string,
    file_manifest: [{path, sha256, bytes}]
  }
]

要求：
- lockfile 变更必须可 diff（JSON 字段顺序固定，数组排序稳定）
- install/fetch 只能使用 lockfile 的 resolved_version

## 3. Overlays

### 3.1 覆盖层级与优先级
最终合成顺序（低 -> 高）：
1) upstream module（store 中）
2) global overlay（repo/overlays/<module_id>/...）
3) project overlay（repo/projects/<project_id>/overlays/<module_id>/...）

### 3.2 overlay 表达形式（v0.1）
采用“文件覆盖”模型：
- overlay 目录结构与 module 一致
- 同路径文件：直接覆盖
- v0.2 可加入 patch/diff 模型（支持 3-way merge）

### 3.3 overlay 编辑命令（见 CLI）
agentpack overlay edit <module_id> [--project] 会：
- 若不存在 overlay：复制 upstream 到 overlay 工作区（仅复制用户选中的文件/或全部，二选一策略见实现）
- 打开编辑器（$EDITOR）
- 保存后 deploy 生效

## 4. CLI 命令（v0.1 必须）

全局参数：
- --repo <path>：指定 config repo 位置
- --profile <name>：默认 default
- --target <name|all>：默认 all
- --json：输出 JSON
- --yes：跳过确认
- --dry-run：只 plan，不 apply（默认 true，除非显式 deploy --apply）

### 4.1 init
agentpack init
- 创建 $AGENTPACK_HOME/repo（可选：初始化 git）
- 写入最小 agentpack.yaml skeleton
- 生成 modules/ 目录

### 4.2 add / remove
agentpack add <type> <source> [--id <id>] [--tags a,b] [--targets codex,claude_code]
agentpack remove <module_id>

source 表达：
- local:modules/xxx
- git:https://...#ref=...&subdir=...

### 4.3 lock
agentpack lock
- 解析所有 modules source
- 生成/更新 lockfile

### 4.4 fetch (install)
agentpack fetch
- 根据 lockfile 把内容拉到 store
- 校验 sha256

### 4.5 plan / diff
agentpack plan
- 输出将要写入哪些 target、哪些文件、何种操作（create/update/delete）
agentpack diff
- 输出逐文件 diff（text），JSON 模式输出 diff 摘要 + 文件 hash 变更

### 4.6 deploy
agentpack deploy [--apply]
默认行为：
- 执行 plan
- 展示 diff
- 若 --apply：执行 apply（带备份）并写 state snapshot
- 若不带 --apply：只展示计划（等价 plan+diff）

### 4.7 status
agentpack status
- 读取目标目录实际内容
- 与“期望状态”（当前 lock + overlays 合成产物）对比
- 输出 drift 列表

### 4.8 rollback
agentpack rollback --to <snapshot_id>
- 恢复备份
- 记录 rollback 事件

### 4.9 bootstrap（AI-first 自举）
agentpack bootstrap [--target codex|claude_code|all] [--scope user|project|both]
- 安装 operator assets：
  - Codex: 写入一个 skill（agentpack-operator）
  - Claude: 写入一组 slash commands（ap-plan/ap-deploy/ap-status/ap-diff）
- 这些 assets 的内容来自 agentpack 内置模板（随版本更新）

要求：
- Claude commands 若含 bash 执行，必须写 allowed-tools（最小化）

## 5. Target Adapter 细则

### 5.1 codex target

Paths（遵循 Codex 文档）：
- codex_home: ~/.codex（可被 CODEX_HOME 覆盖）
- user skills: $CODEX_HOME/skills
- repo skills: 按 Codex skill precedence：
  - $CWD/.codex/skills
  - $CWD/../.codex/skills
  - $REPO_ROOT/.codex/skills
- custom prompts: $CODEX_HOME/prompts（仅 user scope）
- global agents: $CODEX_HOME/AGENTS.md 或 AGENTS.override.md
- repo agents: <repo>/AGENTS.md（也支持 AGENTS.override.md）

部署规则：
- skills：复制目录（不能 symlink）
- prompts：复制 .md 到 prompts 目录
- instructions：
  - global: 渲染 base AGENTS.md 到 $CODEX_HOME/AGENTS.md（或 override 由配置决定）
  - project: 渲染到 repo root AGENTS.md（默认）
  - v0.2 扩展：按路径生成子目录 override

### 5.2 claude_code target（files mode）

Paths：
- repo commands: <repo>/.claude/commands
- user commands: ~/.claude/commands

部署规则：
- command module 是一个 .md 文件，文件名=slash command 名称
- 若 command 内含 !`bash`，必须写 frontmatter allowed-tools: Bash(...)
- v0.1 不强制插件输出；v0.2 可支持 plugin mode（输出 .claude-plugin/plugin.json）

## 6. JSON 输出规范（v0.1）

所有 --json 输出必须包含：
- ok: boolean
- command: string
- version: agentpack version
- data: object
- warnings: [string]
- errors: [ {code, message, details?} ]

plan --json data 示例：
{
  "profile": "work",
  "targets": ["codex", "claude_code"],
  "changes": [
    {
      "target": "codex",
      "op": "update",
      "path": "/home/user/.codex/skills/agentpack-operator/SKILL.md",
      "before_sha256": "...",
      "after_sha256": "...",
      "reason": "module updated"
    }
  ],
  "summary": {"create": 3, "update": 2, "delete": 0}
}

status --json data 示例：
{
  "drift": [
    {"target":"codex","path":"...","expected":"sha256:...","actual":"sha256:...","kind":"modified"}
  ]
}

## 7. 兼容性与限制（v0.1）

- 默认不使用 symlink（除非未来增加 --link 实验开关）
- 不执行第三方 scripts
- prompts 不支持 repo scope（遵循 Codex 文档）；要共享 prompt 请用 skill

## 8. 参考资料
（同 PRD.md）
