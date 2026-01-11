# SPEC.md (v0.4 draft)

> 本文描述 v0.4 计划固化的“契约与抽象”。v0.4 目标是 platformization，因此很多条目属于“行为约束/输出合约”，而不是新增业务功能。

## 0. Versioning / Compatibility

- JSON envelope：`schema_version`（破坏性变更才 bump）
- manifest：`.agentpack.manifest.json` 内 `schema_version`
- lockfile：`agentpack.lock.json` 内 `version`
- config：`agentpack.yaml` 内 `version`

兼容策略：
- v0.4 必须兼容 v0.3 的 repo/lock/manifest（除非提供 migrate）
- 若必须变更：
  - 提供 `agentpack migrate --plan/--apply`（幂等）

## 1. `--json` Guardrails（统一化）

规则：
- 当 `--json` 时，任何会写磁盘或修改 git 的命令必须要求 `--yes`
- 否则：
  - exit code != 0
  - JSON `errors[0].code = "E_CONFIRM_REQUIRED"`

写入命令集合（v0.4 明确列出并保持文档一致）：
- init（含 init --clone）
- add / remove
- lock / fetch / update
- deploy --apply
- rollback --apply（或 rollback 默认 apply 的实现）
- bootstrap
- doctor --fix
- overlay edit（所有 scope）
- remote set / sync
- record
- evolve propose（若会创建 branch/写 overlay）

## 2. JSON Envelope 合约（固定字段）

所有 `--json` 输出必须包含：

```json
{
  "schema_version": 1,
  "ok": true,
  "command": "plan",
  "data": {},
  "warnings": [],
  "errors": []
}
```

错误结构：

```json
{ "code": "E_SOMETHING", "message": "human readable", "details": { } }
```

要求：
- 新增字段只能追加，不允许删除/重命名旧字段（除非 bump schema_version）
- warnings/errors 的 code 枚举需写入文档（至少核心错误）

## 3. 自描述能力（供 AI 学习与编排）

新增/固化：

### 3.1 `agentpack help --json`
输出：
- commands: 命令树（name、args、是否写入、是否支持 --json）
- mutating_commands: 写入命令列表（与 Guardrails 集合一致）
- notes: 推荐 best-practice 顺序（doctor → update → preview → deploy）

### 3.2 `agentpack schema`
输出：
- JSON envelope schema
- 关键命令（plan/diff/preview/status）data 字段说明（最小集合）

## 4. `preview --json --diff` 结构增强

当 `preview --json --diff` 时，data 需要包含：

- plan: summary + changes（沿用现有）
- diff:
  - summary: {create, update, delete, noop}
  - files: [
      {
        "target": "codex",
        "root": "repo_root",
        "path": "AGENTS.md",
        "op": "update",
        "before_hash": "sha256:...",
        "after_hash": "sha256:...",
        "unified": "@@ ...",          // 可选；需 size guard
        "provenance": { ... }         // 可选；来源解释
      }
    ]

size guard：
- unified diff 文本若超过阈值（例如 100KB），应置空并给出 warning（避免 AI 上下文爆炸）

## 5. TargetAdapter 抽象（最低要求）

v0.4 要把新增 target 的最小工作量降到“实现 adapter + 过 conformance tests”。

Adapter 必须能回答：
- roots：哪些目录是 managed roots（每个 root 都写 manifest）
- mapping：把 modules+overlays 的最终文件树映射到 root 下的目标路径
- validation：frontmatter/结构最小校验（例如 Claude command 的 description/allowed-tools）

核心语义由 core enforce：
- delete 只能发生在 manifest.managed_files 内
- manifest 缺失时，delete 必须跳过，并输出 `W_MANIFEST_MISSING_DELETE_SKIPPED`

## 6. Bootstrap operator assets（更可维护）

v0.4 对 operator assets 的要求：
- 覆盖 update/preview/status/explain/evolve propose/deploy 的推荐路径
- 每个 operator 文件写入版本标记（frontmatter 或注释）：
  - `agentpack_version: x.y.z`
- status 可提示“operator assets 版本落后，可运行 bootstrap 更新”
