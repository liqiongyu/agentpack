我先说结论：从仓库首页能看到的“产品形态/命令面/目录结构”来看，你的实现和我们之前讨论的 v0.1 方向基本一致；但如果要严格做一次“代码级别”的验收（例如：overlay 解析算法、lockfile 语义、deploy 的幂等与安全删除、snapshot/rollback 的一致性、targets adapter 的边界等），我这边没法在当前环境里直接把 `src/`、`docs/` 里的文件内容打开来读（GitHub 的目录与文件内容在这个环境里会被动态加载拦住，只能看到仓库首页的 README 摘要和根目录列表）。所以我下面会分两层：

1. 基于仓库首页可见信息，对“是否符合预期”做可验证的判断
2. 按照你接下来的目标（多机器统一管理 + AI-first + 可进化/优化），给出我认为你应该在代码里补齐/增强的点，然后把第二版 PRD/ARCHITECTURE/SPEC/BACKLOG/README 给你（v0.2 规划，能直接指导 `codex cli` 继续开发）

---

基于仓库首页信息，你的实现符合我对 v0.1 的预期的地方

从 repo 根目录列表看，你已经按“产品化工程”落地了：有 `docs/`、`templates/`、`tests/`、`openspec/`、`AGENTS.md`、`CHANGELOG.md`、`Cargo.toml` 等，项目是 Rust 实现。 ([GitHub][1])

从 README 摘要看，产品定位与能力范围也和我们当时的“先管文本/脚本资产（skills/prompts/commands/instructions），MCP 暂时降低优先级”一致：它强调是本地 asset control plane，管理/部署 `AGENTS.md`、skills、Claude Code slash commands、Codex prompts。 ([GitHub][1])

命令面也基本对齐我们讨论过的最小闭环：`init/add/lock/fetch/plan/diff/deploy/status/rollback/bootstrap`，并且 README 给了一个从 init → add → lock/fetch → plan/diff/deploy → status/rollback 的完整路径；同时保留了 `bootstrap` 作为 AI-first 的“操作员资产”安装入口（把 agentpack 自己的 commands/skills 分发到 Claude/Codex）。 ([GitHub][1])

所以：至少在“产品形态”和“用户体验主路径”上，你已经把 v0.1 做出来了，而且命令集合的分层（plan/diff/deploy/status/rollback）是对的，这个非常关键。 ([GitHub][1])

---

我认为现在最需要补齐/优化的点（v0.2 方向）

下面这些点并不意味着你做错了，而是：如果你要让 agentpack 从“单机可用的资产管理 CLI”升级到“多机器一致性 + AI-first + 可进化”的工具，它们会成为你下一阶段的主战场。你可以把它们理解成“产品能力的护城河”。

一、多机器/多项目一致性：把“git 文件夹”从实现细节抬升为“协作模型”
你一开始的理解“全局一个仓库，多台机器基于同一个仓库协同”是最通用的。问题是：如果只停留在“用户自己手动 git push/pull”，就会出现你一开始吐槽的那种麻烦（版本漂移、不同机器不同状态、冲突处理无规范）。

v0.2 我建议把“远端与同步”纳入 agentpack 的一等公民：

* 提供 `agentpack remote` / `agentpack sync`：让用户在 agentpack 里完成 repo 绑定远端、pull/rebase、push、冲突提示（不需要替代 git，但要把“最佳实践路径”固化下来）
* 引入 “machine overlay”（机器级覆盖层）和 “project overlay”（项目级覆盖层）的明确优先级，并且在 `plan` 输出里把每个文件的来源层级解释清楚（可解释性非常重要，否则用户不敢用）

二、Deploy 的安全性与幂等：必须有“只管理我自己产出的文件”的机制
你现在的分发方案是“复制到各自工具的配置目录”，方向没问题；但越往后做，越容易踩到两个坑：

* 你删错了用户原本手写的文件（灾难级体验）
* 你无法判断“目标目录里哪些文件是 agentpack 管的，哪些不是”，导致 drift/status/rollback 不可靠

v0.2 我建议强制引入“管理清单 manifest”机制：

* 每次 deploy 都在目标目录写一个隐藏 manifest（比如 `.agentpack.manifest.json`），记录：managed file list + hash + source module id + snapshot id
* 任何 delete 操作只能删 manifest 里列出的文件
* `status`/`drift` 只对 manifest 中的文件做判断

这会显著提高可靠性，也会让 rollback 可信。

三、AI-first 真正落地：除了 bootstrap，还需要“机器可读 API”
你已经有 `bootstrap`，这是 AI-first 的必要条件之一。 ([GitHub][1])
但要让“AI 能用 agentpack”，你还需要一个稳定的“非交互、可组合”的接口层：

* 所有核心命令支持 `--json` 输出（plan/diff/status/lock/fetch/deploy/rollback），并且 JSON schema 版本化（`schemaVersion`）
* 增加 `agentpack explain <plan|diff|status>`：把复杂输出变成 LLM 友好的“因果解释”（这对 Claude/Codex 的工具调用非常重要）
* 增加 `agentpack doctor`：环境自检，输出“当前机器有哪些 target、路径是否可写、权限是否齐、是否存在冲突文件”等

这样你就可以写 agentpack 自己的 skills/commands，让 AI 先调用 `doctor` → `plan --json` → `diff --json` → `deploy --apply`，形成稳定闭环。

四、“进化/优化”不要一上来做自动改写，而是先做可观测与可评估
你提到的“根据历史调用记录自动优化 skills/prompts/commands”，方向很对，但工程上不要从“自动改写”开始，要从“可观测（logging）+ 可评估（evaluation）+ 人类可控（review）”开始。

我建议 v0.2 的进化能力分三步走：

1. Record（记录）：提供 `agentpack record` 子系统，把 agentpack 自己的 commands/skills 在执行时把“输入/输出摘要、用户反馈、是否成功、耗时”等写到本地 `~/.agentpack/logs/`（或 repo 的 `.agentpack/telemetry/`，可配置）
2. Score（评分）：引入轻量的 “fitness” 概念：例如某个 command 的失败率、被用户手动修改次数、执行后被回滚次数等
3. Propose（提案）：提供 `agentpack evolve propose --module <id>`，生成一个“补丁提案”（patch），但默认不自动 apply；必须走 `diff` + `deploy` 的流程，并留下 snapshot

自动 apply 可以放到 v0.3+，否则你会非常难以建立用户信任。

---

下面给你第二版落地文档（v0.2）

说明：这是“在你 v0.1 已经完成”的前提下，对 v0.2 的升级规划与技术落地 spec。你可以直接把这些文件覆盖到 repo 的 `docs/`（以及根 README），然后让 `codex cli` 按 backlog 开发。

我会额外新增一个文档：`docs/OPERATIONS.md`（多机器/多项目协作的推荐操作手册），因为这件事对“减少用户心智负担”非常关键。

---

```markdown
# PRD.md (v0.2)

## 1. 背景与问题

AI coding 工具生态正在快速分化（Claude Code / Codex CLI / Cursor / 各类 IDE Agents）。用户在不同工具、不同项目、不同机器上需要复用并持续维护一组“Agent 资产”：

- instructions（如 AGENTS.md / project instructions）
- prompts（如 codex prompts）
- commands（如 Claude slash commands / 自定义命令）
- skills（如 SKILL.md + 脚本）
- 可选：MCP（初期降级优先级）

现实痛点：
- 资产来源分散、安装方式不同、升级/回滚困难
- 多项目与多机器之间版本漂移
- 用户对同一个资产会不断手动微调（“进化”需求）
- 工具目录与规范不同，复制粘贴与手工维护成本高且易错

Agentpack 的目标是成为一个本地的“资产控制平面”，用统一抽象管理这些资产，并可安全分发到不同 coding agent 工具及项目中。

## 2. v0.2 目标

### 2.1 产品目标
1) 多机器一致性：同一份资产仓库在多台机器上可一键同步，减少版本漂移。
2) 多项目/多层 overlay：支持 global / machine / project 三层覆盖，且可解释、可预测。
3) 部署安全：deploy 不会误删用户文件；rollback 可信。
4) AI-first 可用：提供稳定的 JSON 输出与自检能力，方便 AI 工具调用 agentpack 完成闭环。
5) 进化的“可观测-可评估-可提案”最小闭环：先记录与评分，再生成可 review 的 patch 提案。

### 2.2 非目标（v0.2 不做）
- 完整的 MCP registry 管理器（只做基础 install/enable/disable 占位或保持现状）
- GUI（保留 CLI + 可选 TUI；GUI 留到生态成熟后）
- 自动无监督地改写用户资产并直接部署到生产（必须经过 diff/review）

## 3. 用户画像与使用场景

### 3.1 主要用户
- 深度 AI coding 用户：多工具、多项目、多机器，依赖 prompts/commands/skills
- 团队 Tech Lead：希望团队共享一套标准化 Agent 资产，并允许项目级定制

### 3.2 核心场景（v0.2）
- S1：新机器初始化（clone 同一份 agentpack repo，一键 bootstrap + deploy）
- S2：新项目接入（继承 global defaults，加 project overlay）
- S3：资产升级/回滚（更新上游 git source，lock/fetch，diff，deploy，必要时 rollback）
- S4：漂移检测（有人手动改了目标目录里的文件，status 能发现并解释）
- S5：进化提案（某个 command/skill 经常失败，系统能基于记录给出可 review 的 patch）

## 4. 关键体验原则

- 可预测：overlay 优先级清晰，plan 输出可解释
- 可回滚：每次 deploy 都可生成 snapshot，并可恢复
- 安全：只操作自己管理的文件（manifest）
- AI 与人一致：AI 调用与人操作走同一套命令与计划/差异模型
- 降低心智负担：默认路径最短（init → sync → plan → deploy）

## 5. 成功指标（v0.2）

- 从空机器到可用：<= 5 条命令完成 bootstrap + deploy（不含安装二进制）
- 99% 情况下 deploy 不会误删用户非托管文件（通过 manifest 机制保障）
- status 输出能定位：哪些文件漂移、来自哪个 overlay/module、预期是什么
- evolve propose 能产出可 review 的 patch（diff 可读），且不会破坏现有 deploy 流程

## 6. 风险与对策

- 风险：多机器同步冲突（git conflicts）
  - 对策：提供 agentpack sync 规范化 pull/rebase/push，冲突时给出清晰提示与建议
- 风险：进化功能导致用户不信任
  - 对策：v0.2 只做 propose，不做自动 apply；所有改动走 diff + snapshot

## 7. 版本规划

- v0.1：已完成基础命令闭环（init/add/lock/fetch/plan/diff/deploy/status/rollback/bootstrap）
- v0.2：remote/sync、manifest 安全、machine overlay、JSON 输出、doctor、record/score/propose
- v0.3：可选 MCP 支持增强、更多 targets（Cursor 等）、可选 TUI、半自动 evolve apply（带守护）
```

---

```markdown
# ARCHITECTURE.md (v0.2)

## 1. 总览

Agentpack 是一个“以本地 git repo 为单一事实来源（Single Source of Truth）”的资产控制平面。

核心思想：
- 所有资产都以 module 的形式存入 agentpack repo（或通过 git source 引入并 lock）
- plan 阶段把 modules + overlays 解析成一个“期望状态 Desired State”
- deploy 阶段将 Desired State 安全地分发到各 target（Claude/Codex/...）
- manifest 记录 deploy 的结果，确保安全删除、可靠 drift/status、可 rollback

## 2. 组件划分

### 2.1 CLI Layer
- clap/arg parsing
- 输出：human readable + --json
- 交互：默认 CLI，预留 TUI（v0.2 可选）

### 2.2 Core Engine
- Config Loader：加载 agentpack.yaml（global + project override）
- Source Manager：处理 local/git sources，lockfile 生成与解析
- Module Resolver：解析模块类型、tag、target filters
- Overlay Resolver：合并 global/machine/project 三层覆盖
- Planner：生成 Plan（期望文件树、每个文件的来源、优先级链）
- Diff Engine：对比计划与当前 manifest/FS，生成 Diff
- Deployer：执行 apply（copy/link），写入 manifest，生成 snapshot
- Status/Drift：基于 manifest 与当前 FS 对比，输出 drift

### 2.3 Storage
- Repo Store：$AGENTPACK_HOME/repo（git 仓库）
- Cache Store：$AGENTPACK_HOME/cache（git sources clone 缓存、下载缓存）
- State Store：$AGENTPACK_HOME/state（lockfile、snapshots、manifests、logs）

### 2.4 Target Adapters
每个 target 有一个 adapter：
- resolve_paths(): 目标目录、文件命名规则
- validate(): 权限/目录存在性/冲突检测
- apply_plan(): 执行 copy/link
- write_manifest(): 写 manifest 到 target scope

### 2.5 AI-first Interface
- --json 输出（稳定 schema）
- explain：将 plan/diff/status 转成可读解释（给人/给 AI）
- bootstrap：安装 agentpack 自带的 commands/skills，用于让 AI 工具调用 agentpack

### 2.6 Evolution Subsystem（v0.2 最小闭环）
- record：写执行记录（由 bootstrap 的 commands/skills 调用）
- score：计算模块健康度（失败率/手改次数/回滚次数）
- propose：生成 patch 提案（默认不 apply）

## 3. 数据模型

### 3.1 Module
- id: string（全局唯一，如 instructions:base）
- kind: instructions|skill|command|prompt|mcp(optional)
- source: local:path | git:url#ref:path
- targets: [claude_code, codex_cli, ...]
- tags: [base, rust, monorepo, ...]
- files: 模块内的文件集合（可由 kind 推导）

### 3.2 Overlay
- scope: global|machine|project
- selectors: tags/targets/project match
- patches: 覆盖文件或参数（文件级覆盖优先）

### 3.3 Plan
- entries: 每个输出文件路径
  - content_hash
  - provenance: [module_id@version, overlay_chain]
  - target: claude_code / codex_cli / ...
  - action: create|update|delete|noop

### 3.4 Manifest
写入 target 目录的 `.agentpack.manifest.json`
- schemaVersion
- generatedAt
- snapshotId
- managedFiles: [{path, hash, moduleId, sourceRef}]
- tool: claude_code|codex_cli|...

### 3.5 Snapshot
- snapshotId
- createdAt
- planHash
- manifests: list
- rollback instructions

## 4. 多机器模型

- 一个“上游 repo”（git remote）作为团队或个人的真源
- 每台机器 clone 到本地 $AGENTPACK_HOME/repo
- machine overlay 存在于 repo（可选择不提交，或提交但按 machineId 分目录）
- agentpack sync 负责 pull/rebase/push 的推荐流程

## 5. 安全与边界

- deploy 删除只能删 manifest 管控的文件
- 任何 target adapter 必须实现 validate，明确目标目录可写
- 记录日志默认本地，不上传；如要上传需显式配置（v0.2 默认不做）

## 6. 可扩展性

- 新工具接入：新增一个 target adapter + 增加 target enum
- 新资产类型接入：新增 kind handler（file mapping / validation）
- evolve 可替换：propose 阶段可对接外部 LLM，也可纯规则
```

---

````markdown
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
````

### 2.2 agentpack.project.yaml（项目根）

```yaml
schemaVersion: 1
profile: default
projectOverlays:
  - id: project:local
    patches:
      - replaceFile:
          target: "claude_code"
          targetPath: ".claude/commands/ap-local.md"
          from: "local:.agentpack/overrides/ap-local.md"
```

## 3. Lockfile

### 3.1 agentpack.lock（state 内）

* 记录所有 git sources 的 resolved commit SHA
* 记录 modules 的 content hash（可选）用于快速判定变更
* schemaVersion 版本化

建议格式（JSON 或 YAML 均可；建议 JSON，利于工具处理）：

```json
{
  "schemaVersion": 1,
  "generatedAt": "2026-01-11T00:00:00Z",
  "sources": {
    "git:https://github.com/foo/bar.git#main:modules/x": {
      "commit": "abc123...",
      "fetchedAt": "..."
    }
  }
}
```

## 4. Manifest

目标目录写入 `.agentpack.manifest.json`：

```json
{
  "schemaVersion": 1,
  "tool": "claude_code",
  "generatedAt": "2026-01-11T00:00:00Z",
  "snapshotId": "snap_20260111_001",
  "managedFiles": [
    {
      "path": ".claude/commands/ap-plan.md",
      "hash": "sha256:...",
      "moduleId": "command:ap-plan",
      "sourceRef": "local:modules/claude-commands/ap-plan.md"
    }
  ]
}
```

约束：

* deploy 删除只能删除 manifest.managedFiles 中出现的 path
* status/drift 的判定以 manifest 为基准（避免扫描用户全目录）

## 5. 命令规范

### 5.1 init

* 初始化 $AGENTPACK_HOME 目录结构
* 初始化 repo（如果不存在）
* 可选：`--clone <remote>` 从远端 clone

### 5.2 add

* `agentpack add <kind> <source> --id <id> [--targets ...] [--tags ...]`
* 修改 repo 内 agentpack.yaml（或写到一个 modules.d/ 目录再合并，取决于实现）

### 5.3 lock / fetch

* lock：解析所有 git sources，写 lockfile（不强制下载）
* fetch：根据 lockfile 拉取/更新 cache，使后续 plan 可离线

### 5.4 plan

* `agentpack plan --profile <name> [--project <path>] [--machine <id>] [--json]`
* 输出 Plan（包含每个目标文件：来源链、hash、action）
* 必须稳定排序（按 target + path）

### 5.5 diff

* 对比 plan 与当前 manifest/FS
* 输出：create/update/delete/noop 列表
* `--json` 输出包含每个文件的 expected hash / actual hash

### 5.6 deploy

* 默认 dry-run（不 apply）
* `agentpack deploy --apply` 执行变更并写 manifest + snapshot
* `--mode copy|symlink` 覆盖 defaults

### 5.7 status

* 基于 manifest 检查 drift
* 输出 drift 的原因：

  * missing（文件被删）
  * changed（hash 不匹配）
  * extra（目标目录里出现非托管文件：只提示不处理）

### 5.8 rollback

* `agentpack rollback --to <snapshotId> [--apply]`
* rollback 也必须走 plan/diff 机制（snapshot 记录当时的 planHash 与 manifest）

### 5.9 bootstrap（AI-first）

* `agentpack bootstrap --target all|claude_code|codex_cli --scope global|project|both`
* 将 agentpack 自带的 operator assets 安装到对应 target
* operator assets 的设计原则：

  * 默认只读/安全（先 plan/diff，再提示 deploy --apply）
  * 优先调用 --json 并把结果解释给用户

### 5.10 doctor（新增 v0.2）

* `agentpack doctor [--json]`
* 检查：

  * AGENTPACK_HOME 可写
  * repo 是否是 git repo
  * targets 路径存在与权限
  * 是否存在 manifest 冲突
  * machineId（生成/展示）

### 5.11 remote / sync（新增 v0.2）

* `agentpack remote set <url>`
* `agentpack sync [--pull-only|--push-only] [--rebase]`
* 输出明确冲突处理建议（不自动解决）

### 5.12 record / score / evolve propose（新增 v0.2）

* `agentpack record <event.json>`（由 commands/skills 调用）
* `agentpack score [--json]` 输出模块健康度
* `agentpack evolve propose --module <id> --goal "<text>" [--json]`

  * 产出 patch 文件（或 git branch）但不自动 apply

## 6. JSON 输出 schema（原则）

* 每个命令的 JSON 输出都有：

  * schemaVersion
  * command
  * generatedAt
  * data: ...
  * warnings: []
  * errors: []

## 7. Targets（v0.2 必须支持）

* claude_code:

  * commands: `.claude/commands/`（project scope）
  * 可选 global scope：用户 home 下某目录（需探测/配置）
* codex_cli:

  * prompts: `~/.codex/prompts/`（global scope）
  * project prompts（如存在）由 adapter 探测/配置

> Cursor / 其他 IDE agents：v0.3+ 以 adapter 形式加入

````

---

```markdown
# BACKLOG.md (v0.2)

> 以 Epic -> Story 的方式组织；每条 Story 需要验收标准（AC）。

## P0（必须做）

### EPIC A：Deploy 安全与幂等（manifest）
A1. 写入 `.agentpack.manifest.json`
- AC:
  - deploy --apply 后目标目录存在 manifest
  - manifest 列出所有 managedFiles，包含 hash/moduleId
A2. 删除保护：deploy 只能删除 manifest 中的文件
- AC:
  - 目标目录存在非托管文件时，deploy 不删除它
  - status 能提示 extra files，但不报错
A3. status/drift 基于 manifest 做校验
- AC:
  - 手动修改托管文件，status 能识别 changed
  - 手动删除托管文件，status 能识别 missing

### EPIC B：多机器同步（remote/sync + machine overlay）
B1. remote set / sync
- AC:
  - 能配置 git remote
  - sync 能执行 pull/rebase/push（至少封装 git 命令并输出提示）
B2. machineId 与 machine overlay
- AC:
  - doctor 输出 machineId
  - plan 能接受 --machine 并应用 machine overlays
  - overlay 生效可解释（plan provenance 里能看到）

### EPIC C：AI-first JSON 输出与 doctor
C1. plan/diff/status/deploy/lock/fetch 支持 --json
- AC:
  - 输出稳定 schemaVersion
  - 内容包含 warnings/errors
C2. doctor 命令
- AC:
  - doctor 输出 target 路径检查结果
  - 对不可写/不存在目录给出清晰建议

## P1（强烈建议做）

### EPIC D：进化最小闭环（record/score/propose）
D1. record：写入事件日志
- AC:
  - 支持从 stdin 读 JSON event 并落盘
  - event 可包含 moduleId、tool、success、feedback
D2. score：计算模块健康度
- AC:
  - 输出每个 module 的失败率、最近修改时间、回滚次数（如可得）
D3. evolve propose：产出 patch 提案（不自动 apply）
- AC:
  - 生成可 review 的 patch 文件（或 git 分支）
  - diff 命令能展示 patch 影响范围

### EPIC E：可解释性增强
E1. explain 子命令：explain plan/diff/status
- AC:
  - 能输出“为什么这个文件来自 overlay X 而不是 Y”的解释

## P2（可后置）

### EPIC F：更多 targets adapter
F1. Cursor adapter（如果 Cursor 有明确的 prompts/commands 目录约定）
F2. VSCode agent 生态适配（视具体工具而定）

### EPIC G：TUI
G1. 交互式查看 plan/diff/status（只读）
G2. deploy/rollback 的交互确认（对人更友好）

### EPIC H：MCP（后置）
H1. agentpack 自己暴露 MCP server（让其他 agent 通过 MCP 调用）
H2. MCP 资源管理增强（registry + enable/disable + version pin）
````

---

````markdown
# README.md (v0.2)

Agentpack is an AI-first local “asset control plane” for managing and deploying agent assets across tools and projects, such as:

- Project instructions (`AGENTS.md`)
- Agent skills (`SKILL.md` + optional scripts)
- Claude Code slash commands (`.claude/commands`)
- Codex custom prompts (`~/.codex/prompts`)

## Why Agentpack

AI coding users typically maintain many prompts/commands/skills that evolve over time.
Agentpack makes them:

- **Versioned**: stored in a local git repo
- **Composable**: global + machine + project overlays
- **Deployable**: to multiple tools with target adapters
- **Safe**: deploy only manages files it owns (manifest), supports rollback
- **AI-first**: stable `--json` outputs and bootstrap assets for tool integration

## Quickstart

```bash
# 1) init repo
agentpack init

# 2) add modules
agentpack add instructions local:modules/instructions/base --id instructions:base --tags base
agentpack add command local:modules/claude-commands/ap-plan.md --id command:ap-plan --tags operator --targets claude_code

# 3) lock & fetch (if you use git sources)
agentpack lock
agentpack fetch

# 4) plan / diff / deploy
agentpack plan --profile default
agentpack diff --profile default
agentpack deploy --profile default --apply

# 5) drift & rollback
agentpack status --profile default
agentpack rollback --to <snapshot_id> --apply
````

## Multi-machine

Recommended workflow:

1. host your `$AGENTPACK_HOME/repo` on a git remote (private is fine)
2. on each machine, clone or init and set remote
3. use `agentpack sync` to pull/rebase/push consistently

## AI-first integration

Install operator assets (commands/skills) that guide agents to use agentpack safely:

```bash
agentpack bootstrap --target all --scope both
```

For agents: prefer `--json` outputs and use this sequence:
`doctor -> plan --json -> diff --json -> deploy --apply`.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

## Contributing

Start with `AGENTS.md` and `CONTRIBUTING.md`.

````

---

```markdown
# OPERATIONS.md (v0.2)

## 推荐的“一个仓库，多台机器”协作方式

### 1) 你应该把什么提交到远端？
建议提交：
- agentpack.yaml（模块声明与 profile）
- modules/（你的全局资产）
- overlays/（可共享的 overlay）
- templates/（生成用模板）
不建议提交：
- state/（lockfile、snapshots、logs 默认不提交，除非你明确要团队共享）
- cache/（机器相关）

### 2) 机器级覆盖怎么做？
两种策略选一种：

A. machine overlay 不入库（更干净）
- machine overlay 写到 `$AGENTPACK_HOME/state/machines/<machineId>.yaml`
- 不参与 git，同步靠你自己的 dotfiles 管理

B. machine overlay 入库（更可控）
- overlays/machines/<machineId>/...
- 提交到 repo，但明确只给自己用

### 3) 项目级覆盖怎么做？
推荐：
- 项目根目录放 `agentpack.project.yaml`
- 里面只写 projectOverlays，不动全局 repo
- 这样项目仓库可以自包含（适合团队协作）

### 4) 发生冲突怎么办？
- 使用 `agentpack sync --rebase`
- 冲突时 agentpack 输出冲突文件列表与建议：
  - 如果冲突发生在 modules：建议人工合并
  - 如果冲突发生在 machine overlay：建议改为不入库策略
````

---

如果你愿意让我做一次“代码级别的严格验收”（例如：manifest 是否实现、lockfile 是否真的 pin 住 commit、deploy 是否绝对不会删非托管文件、overlay 是否 deterministic），我需要能读到仓库里的 `docs/*.md` 和 `src/**/*.rs`。在这个环境里 GitHub 的文件内容打不开，所以你可以任选一个最省事的方式让我继续：

* 方式 A：把关键文件内容直接贴出来（例如：`docs/SPEC.md`、`src/main.rs`、`src/plan.rs`、`src/deploy.rs`）
* 方式 B：给一个可下载的 zip 链接（例如 GitHub 的 `archive/refs/heads/main.zip`）
* 方式 C：把仓库镜像到一个能直接访问 raw 的地址

不管你选哪种，我都可以基于真实实现给你“逐条对照 spec 的 gap list + 具体改动建议 + 更新 v0.2 backlog 的优先级调整”。

在你还没提供源码可读入口之前，以上 v0.2 文档和改动建议，可以先作为下一阶段的开发路线图直接推进。

[1]: https://github.com/liqiongyu/agentpack "GitHub - liqiongyu/agentpack"
