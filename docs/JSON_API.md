# JSON API (the `--json` output contract)

> Current as of **v0.5.0** (2026-01-13). `SPEC.md` is the semantic source of truth; this file focuses on the stable `--json` contract.

## 1) Stability guarantees (principles)

Agentpack’s `--json` output is treated as a programmable API:
- If you pass `--json`, **stdout is always valid JSON** (even on failure; `ok=false` in the envelope).
- `schema_version` is the envelope structure version; current value is `1`.
- For common, actionable failures, `errors[0].code` provides stable error codes (see `ERROR_CODES.md`).
- `warnings` are primarily for human diagnosis; do not rely on string matching for critical branching.

Compatibility policy (`schema_version = 1`):
- **Adding new fields is allowed** (additive; backward-compatible).
- **Removing/renaming fields is not allowed**, and semantics must not change without bumping `schema_version`.

## 2) Envelope shape (`schema_version=1`)

All `--json` outputs include:
- `schema_version`: number
- `ok`: boolean
- `command`: string
- `version`: string (agentpack version)
- `data`: object (success payload; empty object on failure)
- `warnings`: string[]
- `errors`: array[{code,message,details?}]

Failure example:
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

## 3) Mutating guardrails in `--json` mode (must understand)

In `--json` mode, mutating commands require explicit `--yes`, otherwise they return `E_CONFIRM_REQUIRED`.

You can use:
- `agentpack help --json` to obtain the command list and which commands are `mutating`

Common mutating commands (not exhaustive):
- `deploy --apply`, `update`, `lock`, `fetch`, `add/remove`, `bootstrap`, `rollback`
- `overlay edit/rebase`, `doctor --fix`
- `record`, `evolve propose/restore`

## 4) Path field conventions (cross-platform)

To avoid Windows `\` vs POSIX `/` differences forcing heavy branching in automation:
- When a payload includes filesystem paths in `data`, many payloads also include a companion `*_posix` field.
- `*_posix` uses `/` separators and is suitable for cross-platform parsing; the original field remains OS-native for convenience.

Examples: `path` + `path_posix`, `repo` + `repo_posix`, `overlay_dir` + `overlay_dir_posix`.

## 5) Common command payloads (high-level)

Below are the most commonly consumed commands in automation. Field lists focus on stable/high-frequency fields.

### plan

`command = "plan"`

`data`:
- `profile: string`
- `targets: string[]`
- `changes: PlanChange[]`
- `summary: {create, update, delete}`

`PlanChange` fields:
- `target, op(create|update|delete), path, path_posix`
- `before_sha256?, after_sha256?`
- `update_kind? (managed_update|adopt_update)`
- `reason`

### preview

`command = "preview"`

`data`:
- `profile, targets`
- `plan: {changes, summary}`
- Optional: `diff: {changes, summary, files}` (only when `preview --diff --json`)

`diff.files[]`:
- `target, root, root_posix, path, path_posix, op`
- `before_hash?, after_hash?`
- `unified?` (text diff; omitted for large or binary/non-utf8 files with warnings)

### deploy

`command = "deploy"`

`data`:
- `applied: boolean`
- `profile, targets`
- `changes, summary`
- When `applied` is true: `snapshot_id`

Tip:
- If the plan contains `adopt_update`, you must pass `--adopt` or the command returns `E_ADOPT_CONFIRM_REQUIRED` (details include `sample_paths`).

### status

`command = "status"`

`data`:
- `profile, targets`
- `drift: DriftItem[]`
- `summary: {modified, missing, extra}` (additive)

`DriftItem`:
- `target, path, path_posix`
- Optional: `root, root_posix` (additive; target root that contains `path`)
- `expected? (sha256:...)`
- `actual? (sha256:...)`
- `kind: missing|modified|extra`

### overlay.path

`command = "overlay.path"`

`data`:
- `module_id, scope`
- `overlay_dir, overlay_dir_posix`

### evolve.propose (dry-run)

`command = "evolve.propose"`

`data` (when dry-run):
- `created: false`
- `reason: "dry_run"`
- `candidates: [{module_id,target,path,path_posix}]`
- `skipped: [{reason,target,path,path_posix,module_id?,module_ids?,suggestions?}]` (additive)
- `summary: {drifted_proposeable, drifted_skipped, ...}`

`suggestions` (additive):
- `[{action, reason}]`

After execution (non dry-run):
- `created: true`
- `branch, scope, files, files_posix, committed`

## 6) Unstable/fallback code: E_UNEXPECTED

When an error is not classified as a stable UserError, agentpack uses:
- `E_UNEXPECTED`

Do not branch critical automation logic on this; treat it as a “needs human attention” fallback.
