# SPEC.md (v0.2)

> 本文是工程可执行规格：命令、配置、文件格式、算法、输出 schema。

## 1. 目录约定

默认：
- AGENTPACK_HOME: `~/.agentpack`（可用 env 覆盖）
- Repo: `$AGENTPACK_HOME/repo`（git）
- Cache: `$AGENTPACK_HOME/cache`
- State: `$AGENTPACK_HOME/state`
  - lock: `$AGENTPACK_HOME/state/agentpack.lock`
  - snapshots: `$AGENTPACK_HOME/state/snapshots/`
  - logs: `$AGENTPACK_HOME/state/logs/`

项目级：
- 在项目根目录允许存在 `agentpack.project.yaml`（只覆盖 project overlay 与 profile 选择，不直接改全局 repo）

## 2. 配置文件

### 2.1 agentpack.yaml（repo 内）
```yaml
schemaVersion: 1

defaults:
  profile: default
  deployMode: copy   # copy|symlink (v0.2 默认 copy)
  targets: [claude_code, codex_cli]

machines:
  # 可选：机器级覆盖（优先级高于 global，低于 project）
  # machineId 由 `agentpack doctor` 生成/展示（例如 hostname 或自定义）
  my-macbook:
    overlays:
      - id: machine:mac
        tags: [mac]
        vars:
          codexPromptsDir: "~/.codex/prompts"

modules:
  - id: instructions:base
    kind: instructions
    source: "local:modules/instructions/base"
    targets: [all]
    tags: [base]

  - id: command:ap-plan
    kind: command
    source: "local:modules/claude-commands/ap-plan.md"
    targets: [claude_code]
    tags: [base, operator]

profiles:
  default:
    includeTags: [base]
    targetOverrides: {}
    overlays:
      global:
        - id: overlay:global-defaults
          selectors:
            tagsAny: [base]
          patches: []

projects:
  # 可选：project registry（也可不写，改用 agentpack.project.yaml）
  repo-a:
    match:
      pathPrefix: "/path/to/repo-a"
    overlays:
      - id: project:repo-a
        selectors:
          tagsAny: [rust]
        patches:
          - replaceFile:
              targetPath: "AGENTS.md"
              from: "local:overlays/repo-a/AGENTS.md"
