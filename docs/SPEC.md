# SPEC.md

> 本文是项目的**唯一权威 SPEC**（工程可执行、以当前实现为准）。历史快照见 `docs/versions/`。

## 0. 约定

命令名：agentpack
配置 repo：agentpack config repo（本地 clone），默认位于 $AGENTPACK_HOME/repo

默认数据目录：`~/.agentpack`（可通过 `AGENTPACK_HOME` 覆盖），目录结构：
- repo/（config repo, git；含 `agentpack.yaml`、`agentpack.lock.json`）
- cache/（git sources cache）
- state/snapshots/（deploy/rollback snapshots）
- state/logs/（record events）

目前（v0.4）支持：
- target: codex, claude_code
- module types: instructions, skill, prompt, command
- source types: local_path, git (url+ref+subdir)

所有命令默认 human 输出；加 `--json` 输出机器可读 JSON（envelope 包含 `schema_version`、`warnings`、`errors`）。

### 0.1 `--json` 稳定错误码（稳定对外契约）

当 `--json` 启用时，常见/可行动的失败必须返回稳定错误码（`errors[0].code`）：
- `E_CONFIRM_REQUIRED`：在 `--json` 下，写入类命令缺少 `--yes`。
- `E_ADOPT_CONFIRM_REQUIRED`：将覆盖非托管但已存在的文件（adopt_update），但未显式提供 `--adopt`。
- `E_CONFIG_MISSING`：缺少 `repo/agentpack.yaml`。
- `E_CONFIG_INVALID`：`agentpack.yaml` 语法或语义不合法（如缺少 default profile / 重复 module id / source 配置不合法）。
- `E_CONFIG_UNSUPPORTED_VERSION`：`agentpack.yaml` 的 `version` 不被支持。
- `E_LOCKFILE_MISSING`：缺少 `repo/agentpack.lock.json`（但当前命令需要它，如 fetch）。
- `E_LOCKFILE_INVALID`：`agentpack.lock.json` JSON 不合法。
- `E_TARGET_UNSUPPORTED`：不支持的 target（manifest targets 或 CLI `--target` 选择）。
- `E_DESIRED_STATE_CONFLICT`：多个模块对同一 `(target,path)` 产出不同内容（拒绝静默覆盖）。

详见：`docs/ERROR_CODES.md`。

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

```yaml
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
```

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

### 2.3 <target root>/.agentpack.manifest.json（target manifest）

目标：
- 安全删除（只删除托管文件）
- drift/status（changed/missing/extra）

Schema（v1，示例）：

```json
{
  "schema_version": 1,
  "generated_at": "2026-01-11T00:00:00Z",
  "tool": "codex",
  "snapshot_id": "optional",
  "managed_files": [
    { "path": "skills/agentpack-operator/SKILL.md", "sha256": "…", "module_ids": ["skill:agentpack-operator"] }
  ]
}
```

要求：
- `path` 必须是相对路径，且不允许包含 `..`。
- manifest 仅记录 agentpack 本次/历史部署写入的托管文件；不得把用户原生文件视为托管文件。

### 2.4 state/logs/events.jsonl（event log）

`agentpack record` 写入的事件日志为 JSON Lines（每行一个 JSON 对象）。

每行结构（v1，示例）：

```json
{
  "schema_version": 1,
  "recorded_at": "2026-01-11T00:00:00Z",
  "machine_id": "my-macbook",
  "module_id": "command:ap-plan",
  "success": true,
  "event": { "module_id": "command:ap-plan", "success": true }
}
```

约定：
- `event` 为自由 JSON；`score` 仅解析 `module_id|moduleId` 与 `success|ok`。
- 顶层 `module_id` 与 `success` 为可选字段（兼容历史日志）；`score` 会优先使用它们。
- `score` 必须容忍坏行（如截断/非 JSON）：跳过并输出 warning，而不是整个失败。

## 3. Overlays

### 3.1 覆盖层级与优先级
最终合成顺序（低 -> 高）：
1) upstream module（repo 本地目录或 cache 中）
2) global overlay（repo/overlays/<module_fs_key>/...）
3) machine overlay（repo/overlays/machines/<machine_id>/<module_fs_key>/...）
4) project overlay（repo/projects/<project_id>/overlays/<module_fs_key>/...）

其中：
- `module_fs_key` 是从 `module_id` 派生的、跨平台安全的目录名（会做字符规整，并附带短 hash 以避免碰撞）。
- CLI 与 manifest 中仍使用原始 `module_id`；`module_fs_key` 仅用于磁盘寻址。

### 3.2 overlay 表达形式（v0.2）
采用“文件覆盖”模型：
- overlay 目录结构与 module 一致
- 同路径文件：直接覆盖
- （future）可加入 patch/diff 模型（如 3-way merge），但当前实现未支持

### 3.3 overlay 编辑命令（见 CLI）
agentpack overlay edit <module_id> [--scope global|machine|project] 会：
- 若不存在 overlay：复制 upstream module 的完整文件树到 overlay 目录（scope 对应路径见下）
- 打开编辑器（$EDITOR）
- 保存后 deploy 生效

scope → 路径映射：
- global: `repo/overlays/<module_fs_key>/...`
- machine: `repo/overlays/machines/<machine_id>/<module_fs_key>/...`
- project: `repo/projects/<project_id>/overlays/<module_fs_key>/...`

兼容性：
- `--project` 仍保留但已 deprecated（等价 `--scope project`）。

补充（v0.3+）：
- `agentpack overlay path <module_id> [--scope global|machine|project]`
  - human：打印 overlay 目录绝对路径
  - json：`data.overlay_dir`

### 3.4 overlay 元数据（.agentpack）
- overlay skeleton 会写入 `<overlay_dir>/.agentpack/baseline.json`，用于 overlay drift warnings（不参与部署）。
- `.agentpack/` 目录为保留元数据目录：不会被 deploy 到 target roots；也不应被写入模块产物中。

## 4. CLI 命令（v0.2）

全局参数：
- --repo <path>：指定 config repo 位置
- --profile <name>：默认 default
- --target <name|all>：默认 all
- --machine <id>：指定 machine overlay（默认自动探测 machineId）
- --json：输出 JSON
- --yes：跳过确认
- --dry-run：强制不写入（即使 `deploy --apply`）；默认 false

安全约定：
- 对会写入磁盘/改写 git 的命令，`--json` 模式下要求同时提供 `--yes`（避免 LLM/脚本误触写入）。
- 若缺少 `--yes`：退出码非 0，stdout 仍为合法 JSON（`ok=false`），并返回稳定错误码 `E_CONFIRM_REQUIRED`（`errors[0].code`）。

### 4.1 init
agentpack init
- 创建 $AGENTPACK_HOME/repo（不会自动 `git init`）
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
- 根据 lockfile 把内容拉到 cache（git sources checkout）
- 校验 sha256

v0.3+ 行为增强（减少脚枪）：
- 当 lockfile 存在且某个 `<moduleId, commit>` 的 checkout 缓存缺失时，`plan/diff/deploy/overlay edit` 会自动补齐缺失 checkout（安全网络操作），不再强制要求用户先手动 `fetch`。

### 4.4.1 update（组合命令）
agentpack update [--lock] [--fetch] [--no-lock] [--no-fetch]
- 默认策略：
  - 若 lockfile 不存在：执行 lock + fetch
  - 若 lockfile 已存在：默认只执行 fetch
- 用途：减少常见链路（lock/fetch）摩擦，便于 AI/脚本编排。

补充：
- `--json` 模式下属于写入命令：要求同时提供 `--yes`（缺少则 `E_CONFIRM_REQUIRED`）。
- `--json` 输出会聚合 steps：`data.steps=[{name, ok, detail}, ...]`。

### 4.4.2 preview（组合命令）
agentpack preview [--diff]
- 总是执行 plan
- 当 `--diff` 时额外输出 diff（human：unified diff；json：diff 摘要）

补充：
- preview 为纯读取操作，不需要 `--yes`。

### 4.5 plan / diff
agentpack plan
- 输出将要写入哪些 target、哪些文件、何种操作（create/update/delete）
- 若多个模块对同一 `(target,path)` 产出：
  - 同路径同内容：合并 module_ids（用于 provenance/解释）
  - 同路径不同内容：报错并返回 `E_DESIRED_STATE_CONFLICT`（默认阻止 apply）
agentpack diff
- 输出逐文件 diff（text），JSON 模式输出 diff 摘要 + 文件 hash 变更
- 对 update 操作：JSON 会额外输出 `update_kind`（`managed_update` / `adopt_update`）。

### 4.6 deploy
agentpack deploy [--apply] [--adopt]
默认行为：
- 执行 plan
- 展示 diff
- 若 --apply：执行 apply（带备份）并写 state snapshot；并写入每个 target root 的 `.agentpack.manifest.json`
- 删除保护：仅删除 manifest 中记录的托管文件（不会删除用户非托管文件）
- 覆盖保护：默认拒绝覆盖“非托管但已存在”的文件（adopt_update），除非显式提供 `--adopt`
- 若不带 --apply：只展示计划（等价 plan+diff）

补充：
- `--json` + `--apply` 必须同时提供 `--yes`，否则报错（`E_CONFIRM_REQUIRED`）。
- 若 plan 中包含 adopt_update：apply 必须提供 `--adopt`；`--json` 模式下缺少则返回 `E_ADOPT_CONFIRM_REQUIRED`。
- 即使 plan 为空，只要目标 root 缺失 manifest，也会写入 manifest（保证后续 drift/safe-delete 可用）。

### 4.7 status
agentpack status
- 若目标目录存在 `.agentpack.manifest.json`：基于 manifest 做 drift（changed/missing/extra）
- 若没有 manifest（首次升级/未部署）：降级为对比 desired vs FS，并提示 warning
- 若检测到已安装的 operator assets（bootstrap）缺失或版本落后：提示 warning，并建议运行 `agentpack bootstrap` 更新

### 4.8 rollback
agentpack rollback --to <snapshot_id>
- 恢复备份
- 记录 rollback 事件

### 4.9 bootstrap（AI-first 自举）
agentpack bootstrap [--target codex|claude_code|all] [--scope user|project|both]
- 安装 operator assets：
  - Codex: 写入一个 skill（agentpack-operator）
  - Claude: 写入一组 slash commands（ap-doctor/ap-update/ap-preview/ap-plan/ap-diff/ap-deploy/ap-status/ap-explain/ap-evolve）
- 这些 assets 的内容来自 agentpack 内置模板（随版本更新）
- 每个 operator 文件写入版本标记：`agentpack_version: x.y.z`（frontmatter 或注释）

要求：
- Claude commands 若含 bash 执行，必须写 allowed-tools（最小化）

补充：
- `--json` 模式下要求同时提供 `--yes`（因为会写入目标目录；缺少则 `E_CONFIRM_REQUIRED`）。

### 4.10 doctor
agentpack doctor [--fix]
- 输出 machineId（用于 machine overlays）
- 检查并报告 target roots 是否存在/可写，并给出建议（mkdir/权限/配置）
- Git hygiene（v0.3+）：
  - 若某个 target root 位于 git repo 内，且 `.agentpack.manifest.json` 未被 ignore：输出 warning（避免误提交）。
  - `--fix`：幂等地向对应 git repo 的 `.gitignore` 追加一行：`.agentpack.manifest.json`。
    - `--json` 下若会写入，需要同时提供 `--yes`（缺少则 `E_CONFIRM_REQUIRED`）。

### 4.11 remote / sync
agentpack remote set <url> [--name origin]
agentpack sync [--rebase] [--remote origin]
- 用 git 命令封装推荐的多机器同步流程（pull/rebase + push）
- 不自动解决冲突；遇到冲突直接报错并提示用户处理

### 4.12 record / score
agentpack record   # 从 stdin 读取 JSON 并写入 state/logs/events.jsonl
agentpack score    # 根据 events.jsonl 计算 module 失败率等指标

事件字段约定（v0.2）：
- `record` 读取 stdin 的 JSON 为 `event`（不做强 schema 限制）。
- `score` 识别：
  - module id：`module_id` 或 `moduleId`
  - success：`success` 或 `ok`（缺省视为 true）

### 4.13 explain
agentpack explain plan|diff|status
- 输出变更/漂移的“来源解释”：moduleId + overlay layer（project/machine/global/upstream）

### 4.14 evolve propose
agentpack evolve propose [--module-id <id>] [--scope global|machine|project]
- 捕获 drifted 的已部署文件内容，生成 overlay 变更（在 config repo 创建 proposal branch；不自动 deploy）

补充：
- `--json` 模式下要求同时提供 `--yes`（缺少则 `E_CONFIRM_REQUIRED`）。
- 要求 config repo 工作树干净；会创建分支并尝试提交（若 git identity 缺失导致提交失败，会提示并保留分支与改动）。
- 当前实现是保守的（conservative），只会对“可安全映射回单个模块”的 drift 生成 proposal：
  - 仅处理 `module_ids.len()==1` 的输出（多模块聚合输出会被跳过并提示）。
  - 仅处理 deployed 文件“存在且内容不同”的 drift；对于 `missing` drift（文件不存在）会跳过并提示（建议通过 `deploy` 恢复）。
  - 推荐先用 `agentpack evolve propose --dry-run --json` 查看 candidates / skipped / warnings，再决定是否 `--yes` 创建 proposal 分支。

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
- global agents: $CODEX_HOME/AGENTS.md
- repo agents: <repo>/AGENTS.md

部署规则：
- skills：复制目录（不能 symlink）
- prompts：复制 .md 到 prompts 目录
- instructions：
  - global: 渲染 base AGENTS.md 到 $CODEX_HOME/AGENTS.md
  - project: 渲染到 repo root AGENTS.md（默认）
  - （future）更细粒度的 subdir override

### 5.2 claude_code target（files mode）

Paths：
- repo commands: <repo>/.claude/commands
- user commands: ~/.claude/commands

部署规则：
- command module 是一个 .md 文件，文件名=slash command 名称
- 若 command 内含 !`bash`，必须写 frontmatter allowed-tools: Bash(...)
- （future）可支持 plugin mode（输出 .claude-plugin/plugin.json），但当前实现未支持

## 6. JSON 输出规范（v0.2）

详见：`docs/JSON_API.md`。

所有 --json 输出必须包含：
- schema_version: number
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
	      "update_kind": "managed_update",
	      "reason": "content differs"
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

## 7. 兼容性与限制（v0.2）

- 默认不使用 symlink（除非未来增加 --link 实验开关）
- 不执行第三方 scripts
- prompts 不支持 repo scope（遵循 Codex 文档）；要共享 prompt 请用 skill

## 8. 参考资料
（同 PRD.md）
