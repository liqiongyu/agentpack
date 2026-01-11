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
