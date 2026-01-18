# Troubleshooting

> Language: English | [Chinese (Simplified)](zh-CN/TROUBLESHOOTING.md)

This document is organized as **symptom → cause → fix**, and aims to use stable error codes whenever possible.

If you use `--json`, prioritize:
- `errors[0].code`
- `errors[0].details` (if present)

See `ERROR_CODES.md` for the full registry.

## 1) E_CONFIRM_REQUIRED

Symptom:
- Mutating commands fail in `--json` mode
- Error code: `E_CONFIRM_REQUIRED`

Cause:
- Agentpack treats `--json` as a machine API. To prevent accidental writes from scripts/LLMs, mutating commands require explicit `--yes`.

Fix:
- Confirm you truly intend to write, then retry with `--yes`:
  - `agentpack --json deploy --apply --yes`
  - `agentpack --json update --yes`
  - `agentpack --json bootstrap --yes`

## 2) E_ADOPT_CONFIRM_REQUIRED

Symptom:
- `deploy --apply` refuses to overwrite some files
- Error code: `E_ADOPT_CONFIRM_REQUIRED`

Cause:
- These updates are `adopt_update`: the destination file exists but is not in agentpack’s managed manifest. It is refused by default.

Fix:
1) Preview and confirm the impact:
- `agentpack preview --diff`

2) If you want to take over and overwrite:
- `agentpack deploy --apply --adopt`

Tip:
- `errors[0].details.sample_paths` usually includes a sample list of affected paths.

## 3) E_DESIRED_STATE_CONFLICT

Symptom:
- `plan/preview/deploy` reports “conflicting desired outputs...`
- Error code: `E_DESIRED_STATE_CONFLICT`

Cause:
- Multiple modules produced different content for the same `(target, path)`. Agentpack refuses to silently overwrite.

Fix:
- Adjust the manifest so the path is produced by only one module, or ensure the content is identical.
- If caused by overlays, check whether an overlay file is unintentionally overriding another module’s output.

## 4) E_CONFIG_MISSING / E_CONFIG_INVALID / E_CONFIG_UNSUPPORTED_VERSION

Symptom:
- `agentpack.yaml` is missing or cannot be parsed

Fix:
- `E_CONFIG_MISSING`: run `agentpack init` or point to the correct repo via `--repo`
- `E_CONFIG_INVALID`: fix YAML based on `details.error`; note `profiles.default` is required
- `E_CONFIG_UNSUPPORTED_VERSION`: set manifest `version` to a supported value (currently `1`) or upgrade agentpack

## 5) E_LOCKFILE_MISSING / E_LOCKFILE_INVALID

Symptom:
- `fetch` (or commands that need git modules) report missing/invalid lockfile

Fix:
- Generate: `agentpack lock` or `agentpack update`
- If corrupted: delete `agentpack.lock.json`, then run `agentpack update` again

## 6) E_TARGET_UNSUPPORTED

Symptom:
- `--target` uses an unsupported value, or the manifest config contains an unknown target

Fix:
- `--target` must be `all|codex|claude_code|cursor|vscode|jetbrains|zed`
- Manifest targets must be one of: `codex|claude_code|cursor|vscode|jetbrains|zed`

## 7) Overlay errors

### E_OVERLAY_NOT_FOUND
- Run `agentpack overlay edit <module_id>` to create the overlay.

### E_OVERLAY_BASELINE_MISSING
- Overlay metadata is missing (often due to manual directory creation).
- Fix: re-run `agentpack overlay edit <module_id>` to regenerate `.agentpack/baseline.json`.

### E_OVERLAY_BASELINE_UNSUPPORTED
- The baseline cannot locate a merge base (e.g., baseline too old or upstream identity missing).
- Fix: recreate the overlay (new baseline), or switch upstream to a traceable git source and recreate the overlay.

### E_OVERLAY_REBASE_CONFLICT
- 3-way merge conflicts during `overlay rebase`.
- Fix: open conflict-marked files under the overlay directory, resolve, then re-run rebase (or commit overlay changes manually).

## 8) evolve propose fails: dirty working tree

Symptom:
- `evolve propose` reports “working tree dirty” / “refusing to propose...”

Cause:
- Propose creates a branch and writes overlay files. It requires a clean config repo working tree to avoid mixing unrelated changes.

Fix:
- Run `git status`, then commit or stash changes
- Retry `agentpack evolve propose ...`

## 9) Windows path issues

Symptom:
- Module ids containing `:` fail to create overlay directories

Notes:
- Agentpack uses `module_fs_key` (sanitize + hash) as directory names to avoid Windows invalid characters.
- If migrating from older versions, legacy overlay directory names may be incompatible. Recommended:
  - Use `agentpack overlay path <module_id>` to see the effective directory
  - Keep only one naming scheme to avoid confusion (legacy + canonical coexisting)
