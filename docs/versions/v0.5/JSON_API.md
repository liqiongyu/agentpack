# JSON_API.md

> Current as of **v0.5** (2026-01-12). `docs/SPEC.md` remains the authoritative contract.

## 1. 稳定性承诺（原则）

Agentpack 的 `--json` 输出被视为可编排的 API：
- **stdout 永远是合法 JSON**（即使失败也返回 envelope，`ok=false`）。
- `schema_version` 表示 envelope 的结构版本；当前为 `1`。
- `warnings` 面向人类诊断（允许变动），**不要**依赖字符串匹配做关键分支。
- 对常见且可行动的失败，`errors[0].code` 提供稳定错误码（见 `docs/ERROR_CODES.md`）。

兼容性（schema_version=1）：
- **允许新增字段**（additive change）作为向后兼容演进。
- **不允许删除/重命名字段** 或改变字段语义而不升级 `schema_version`（breaking change）。

## 2. Envelope 结构（schema_version=1）

所有 `--json` 输出都包含：
- `schema_version`: number
- `ok`: boolean
- `command`: string
- `version`: string（agentpack 版本号）
- `data`: object（成功返回的 payload；失败时为默认空值）
- `warnings`: string[]
- `errors`: {code, message, details?}[]

示例（失败）：
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

## 3. 版本化与演进建议

- 自动化逻辑应以 `schema_version` + `errors[0].code` 为主。
- 对 `data` 的解析建议采用“可选字段 + 默认值”的方式，容忍未来新增字段。
- 对 `warnings` 仅做展示或日志记录，不建议做强逻辑分支。

## 4. 路径字段约定（跨平台）

为避免 Windows `\` 与 POSIX `/` 的差异导致 automation 需要大量特判：
- 当 `data` 中包含文件系统路径字段（如 `path` / `root` / `repo` / `overlay_dir` / `lockfile` / `store` 等）时，agentpack 会额外输出对应的 `*_posix` 字段。
- `*_posix` 使用 `/` 作为分隔符，便于跨平台解析；原字段保持原样（native），用于本机直接访问文件系统更方便。
- 该约定是 additive change：`schema_version` 仍为 `1`。
