# JSON API（--json 输出契约）

> Current as of **v0.5.0** (2026-01-13). `SPEC.md` 是更“语义级”的权威说明；本文件聚焦 `--json` 的稳定契约。

## 1) 稳定性承诺（原则）

Agentpack 的 `--json` 输出被视为可编排 API：
- 只要你传了 `--json`，**stdout 永远是合法 JSON**（即使失败也会返回 envelope，`ok=false`）。
- `schema_version` 表示 envelope 结构版本；当前为 `1`。
- 对常见且可行动的失败，`errors[0].code` 提供稳定错误码（见 `ERROR_CODES.md`）。
- `warnings` 主要用于人类诊断：允许优化调整，不建议依赖字符串匹配做关键分支。

兼容性策略（schema_version = 1）：
- **允许新增字段**（additive）作为向后兼容演进。
- **不允许删除/重命名字段** 或改变字段语义而不升级 `schema_version`。

## 2) Envelope 结构（schema_version=1）

所有 `--json` 输出都包含：
- `schema_version`: number
- `ok`: boolean
- `command`: string
- `version`: string（agentpack 版本号）
- `data`: object（成功 payload；失败时为默认空值）
- `warnings`: string[]
- `errors`: array[{code,message,details?}]

失败示例：
```json
{
  "schema_version": 1,
  "ok": false,
  "command": "deploy",
  "version": "0.5.0",
  "data": {},
  "warnings": [],
  "errors": [
    {
      "code": "E_CONFIRM_REQUIRED",
      "message": "refusing to run 'deploy --apply' in --json mode without --yes",
      "details": {"command": "deploy --apply"}
    }
  ]
}
```

## 3) `--json` 下的写入保护（必须理解）

在 `--json` 模式下，写入类命令必须显式 `--yes`，否则返回 `E_CONFIRM_REQUIRED`。

你可以用：
- `agentpack help --json` 获取命令列表与“哪些命令属于 mutating（写入）”

常见写入类命令（不完整）：
- `deploy --apply`、`update`、`lock`、`fetch`、`add/remove`、`bootstrap`、`rollback`
- `overlay edit/rebase`、`doctor --fix`
- `record`、`evolve propose/restore`

## 4) 路径字段约定（跨平台）

为避免 Windows `\` 与 POSIX `/` 的差异导致 automation 需要大量特判：
- 当 `data` 中出现文件系统路径字段时，很多 payload 会同时给一个 `*_posix` 伴随字段。
- `*_posix` 使用 `/` 分隔符，适合跨平台解析；原字段保留 OS-native 形式，便于本机直接访问。

例：`path` + `path_posix`，`repo` + `repo_posix`，`overlay_dir` + `overlay_dir_posix`。

## 5) 常用命令的 data 结构（要点版）

下面列出自动化最常用的一批命令，字段以“稳定/高频”优先。

### plan

`command = "plan"`

`data`：
- `profile: string`
- `targets: string[]`
- `changes: PlanChange[]`
- `summary: {create, update, delete}`

`PlanChange`（字段）：
- `target, op(create|update|delete), path, path_posix`
- `before_sha256?, after_sha256?`
- `update_kind? (managed_update|adopt_update)`
- `reason`

### preview

`command = "preview"`

`data`：
- `profile, targets`
- `plan: {changes, summary}`
- 可选：`diff: {changes, summary, files}`（仅当 `preview --diff --json`）

`diff.files[]`：
- `target, root, root_posix, path, path_posix, op`
- `before_hash?, after_hash?`
- `unified?`（文本 diff，过大/二进制会被省略并用 warnings 说明）

### deploy

`command = "deploy"`

`data`：
- `applied: boolean`
- `profile, targets`
- `changes, summary`
- 当 applied 为 true：`snapshot_id`

提示：
- 若计划包含 `adopt_update`，必须加 `--adopt`，否则返回 `E_ADOPT_CONFIRM_REQUIRED`（details 会给 sample_paths）。

### status

`command = "status"`

`data`：
- `profile, targets`
- `drift: DriftItem[]`

`DriftItem`：
- `target, path, path_posix`
- `expected? (sha256:...)`
- `actual? (sha256:...)`
- `kind: missing|modified|extra`

### overlay.path

`command = "overlay.path"`

`data`：
- `module_id, scope`
- `overlay_dir, overlay_dir_posix`

### evolve.propose（dry-run）

`command = "evolve.propose"`

`data`（dry-run 时）：
- `created: false`
- `reason: "dry_run"`
- `candidates: [{module_id,target,path,path_posix}]`
- `skipped: [{reason,target,path,path_posix,module_id?,module_ids?}]`
- `summary: {drifted_proposeable, drifted_skipped, ...}`

执行（非 dry-run）后：
- `created: true`
- `branch, scope, files, files_posix, committed`

## 6) 不稳定错误码：E_UNEXPECTED

当错误没有被归类为稳定 UserError 时，会用：
- `E_UNEXPECTED`

这类错误不建议做强逻辑分支；更适合作为“需要人类介入”的兜底。
