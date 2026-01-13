# 配置文件与模块（agentpack.yaml）

Agentpack 的“单一真源”是 config repo 里的 `agentpack.yaml`。

你也可以手改 YAML，但更推荐用 `agentpack add/remove` 来改（能自动校验、少踩坑）。

## 文件位置

默认：`$AGENTPACK_HOME/repo/agentpack.yaml`（`AGENTPACK_HOME` 默认 `~/.agentpack`）

也可用 `agentpack --repo <path>` 指定。

## 最小示例

```yaml
version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  codex:
    mode: files
    scope: both
    options:
      codex_home: "~/.codex"
      write_repo_skills: true
      write_user_skills: true
      write_user_prompts: true
      write_agents_global: true
      write_agents_repo_root: true
  claude_code:
    mode: files
    scope: both
    options:
      write_repo_commands: true
      write_user_commands: true
      write_repo_skills: false
      write_user_skills: false

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    source:
      local_path:
        path: "modules/instructions/base"
```

## 字段说明

### version

- 当前支持：`1`

### profiles

Profile 用来从一堆 modules 里筛选“本次要部署哪些”。

字段：
- `include_tags: [string]`：包含这些 tags 的模块
- `include_modules: [module_id]`：显式包含模块
- `exclude_modules: [module_id]`：显式排除模块

建议：
- 至少有一个 `default` profile（必需）

### targets

目前内置 targets：
- `codex`
- `claude_code`

每个 target 的字段：
- `mode`: 目前只有 `files`
- `scope`: `user|project|both`
- `options`: target-specific 的 key/value（YAML 任意值）

注意：
- `scope` 会影响哪些 roots 会被写入（例如 user 目录 / project 目录）。

### modules

每个 module 的字段：
- `id: string`：全局唯一，建议 `type:name`（例如 `skill:git-review`）
- `type: instructions|skill|prompt|command`
- `enabled: bool`：默认 true
- `tags: [string]`：用于 profiles
- `targets: [string]`：限制仅对某些 target 生效；空数组 = all
- `source`: 见下
- `metadata: {k: v}`：可选（纯透传，便于写注释/描述）

#### source（两种）

1) local_path

```yaml
source:
  local_path:
    path: "modules/instructions/base"
```

约定：
- 建议使用 repo 内相对路径。

2) git

```yaml
source:
  git:
    url: "https://github.com/your-org/agentpack-modules.git"
    ref: "v1.2.0"      # tag/branch/commit；默认 main
    subdir: "skills/git-review"   # 可空
    shallow: true       # 默认 true
```

说明：
- git sources 会被 lock 到具体 commit（写进 `agentpack.lock.json`），确保可复现。

## module 类型约束（重要）

Agentpack 会在渲染前验证每个 module 的结构：

- `instructions`：必须包含 `AGENTS.md`
- `skill`：必须包含 `SKILL.md`
- `prompt`：必须“最终只有一个 `.md` 文件”
- `command`：必须“最终只有一个 `.md` 文件”，且必须包含 YAML frontmatter：
  - 必需字段：`description`
  - 若正文里使用 `!bash`/`!\`bash\``：frontmatter 必须包含 `allowed-tools` 并允许 `Bash(...)`

提示：prompt/command 的 source 可以是单文件，也可以是一个目录；但 materialize 后必须只剩 1 个文件。

更多：
- targets 具体写入规则见 `TARGETS.md`
- overlays 与 source 合成规则见 `OVERLAYS.md`
