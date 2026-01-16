# ERROR_CODES.md (stable error code registry)

> Current as of **v0.6.0** (2026-01-15). `SPEC.md` is the semantic source of truth; this file is the stable registry for `--json` automation (`errors[0].code`).

This file defines stable, externally-consumable error codes for `--json` mode (`errors[0].code`).

Conventions:
- When `ok=false`, the process exit code is non-zero.
- `errors[0].code` is for automation branching; `errors[0].message` is primarily for humans (may be refined over time).
- Do not branch critically on `warnings` (strings are not stable).

## Stable error codes

### E_CONFIRM_REQUIRED
Meaning: in `--json` mode, the command would perform a mutation (filesystem and/or git), but `--yes` is missing.
Typical cases: `deploy --apply --json`, `update --json`, `overlay edit --json`, etc.
Retryable: yes.
Recommended action: confirm you intend to write, then retry with `--yes`, or drop `--json` and use interactive confirmation.
Details: usually includes `{"command": "..."}`.

### E_ADOPT_CONFIRM_REQUIRED
Meaning: `deploy --apply` would overwrite an existing unmanaged file (`adopt_update`), but `--adopt` was not provided.
Retryable: yes.
Recommended action:
- Run `preview --diff` to confirm scope/impact.
- If you truly want to take over and overwrite, retry with `--adopt`.
Details: includes `{flag, adopt_updates, sample_paths}`.

### E_IMPORT_CONFLICT
Meaning: `import --apply` would overwrite an existing path in the config repo (module file/dir destination already exists), and Agentpack refused to overwrite.
Retryable: yes.
Recommended action: delete or move the conflicting destination paths, then re-run `agentpack import --apply`.
Details: includes `{count, sample_paths, sample_paths_posix, hint}`.

### E_CONFIG_MISSING
Meaning: missing `repo/agentpack.yaml`.
Retryable: yes.
Recommended action: run `agentpack init` to create a skeleton, or point to the correct repo via `--repo`.
Details: typically includes `{path, hint}`.

### E_CONFIG_INVALID
Meaning: `agentpack.yaml` is syntactically or semantically invalid.
Retryable: depends on fixing config.
Recommended action: fix YAML based on `details` and/or error message (e.g., missing default profile, duplicate module id, invalid source, missing target config).

This code MAY also be used when a configured module is structurally invalid (e.g., a `skill` module’s `SKILL.md` has missing/invalid YAML frontmatter).

### E_CONFIG_UNSUPPORTED_VERSION
Meaning: `agentpack.yaml` `version` is unsupported.
Retryable: depends on fixing config or upgrading agentpack.
Recommended action: set `version` to a supported value (currently `1`) or upgrade agentpack.
Details: typically includes `{version, supported}`.

### E_LOCKFILE_MISSING
Meaning: missing `repo/agentpack.lock.json` but the command requires it (e.g., `fetch`).
Retryable: yes.
Recommended action: run `agentpack lock` or `agentpack update`.

### E_LOCKFILE_INVALID
Meaning: `agentpack.lock.json` is invalid JSON or cannot be parsed.
Retryable: depends on repair/rebuild.
Recommended action: fix JSON or delete it and regenerate via `agentpack update`.

### E_LOCKFILE_UNSUPPORTED_VERSION
Meaning: `agentpack.lock.json` `version` is unsupported.
Retryable: depends on upgrading agentpack or regenerating lockfile.
Recommended action: upgrade agentpack, or regenerate the lockfile via `agentpack lock` / `agentpack update`.
Details: typically includes `{version, supported}`.

### E_TARGET_UNSUPPORTED
Meaning:
- `--target` specifies an unsupported value, or
- The manifest config contains an unknown target.
- The target is not compiled into the running agentpack binary (feature-gated builds).
Retryable: yes.
Recommended action:
- `--target` must be `all|codex|claude_code|cursor|vscode` (but feature-gated builds may support a subset; see `agentpack help --json` `data.targets[]`).
- Manifest targets must be built-in targets that are compiled into the running binary.

### E_DESIRED_STATE_CONFLICT
Meaning: multiple modules produced different content for the same `(target, path)`. Agentpack refuses to silently overwrite.
Retryable: depends on config/overlay fixes.
Recommended action: adjust modules/overlays so only one module produces that path, or make the contents identical.
Details: includes both sides’ sha256 and module_ids.

### E_OVERLAY_NOT_FOUND
Meaning: requested overlay directory does not exist.
Retryable: yes.
Recommended action: run `agentpack overlay edit <module_id>` to create the overlay.

### E_OVERLAY_BASELINE_MISSING
Meaning: overlay metadata is missing (`<overlay_dir>/.agentpack/baseline.json`), so rebase cannot proceed.
Retryable: yes.
Recommended action: re-run `agentpack overlay edit <module_id>` to regenerate metadata.

### E_OVERLAY_BASELINE_UNSUPPORTED
Meaning: overlay baseline cannot locate a merge base, so rebase cannot proceed safely.
Retryable: depends on baseline repair.
Recommended action: usually recreate the overlay (new baseline), or ensure upstream is traceable (git) and recreate.

### E_OVERLAY_REBASE_CONFLICT
Meaning: `overlay rebase` produced conflicts that cannot be auto-merged.
Retryable: yes (after resolving conflicts).
Recommended action: open the conflict-marked files under the overlay directory (for patch overlays: `.agentpack/conflicts/<relpath>`), resolve, then re-run `agentpack overlay rebase` (or commit overlay changes directly).
Details: includes `{conflicts, summary, overlay_dir, scope, ...}`.

### E_OVERLAY_PATCH_APPLY_FAILED
Meaning: patch overlay application failed during desired-state generation (the patch could not be applied cleanly).
Retryable: yes (after regenerating/fixing the patch).
Recommended action:
- regenerate the patch against current upstream (or lower overlays) content, or
- switch to a directory overlay for that file.
Details: includes `{module_id, scope, overlay_dir, patch_file, relpath, stderr, ...}`.

### E_POLICY_VIOLATIONS
Meaning: `policy lint` detected one or more governance policy violations.
Retryable: yes (after fixing the violations).
Recommended action:
- Run `agentpack policy lint --json` to get machine-readable issues (suitable for CI gating).
- Fix the reported issues and rerun until `ok=true`.
Details: includes `{root, root_posix, issues, summary}` where `issues[]` items include `{rule, path, path_posix, message, details?}`.

### E_POLICY_CONFIG_MISSING
Meaning: missing `repo/agentpack.org.yaml` when running governance policy commands that require it (e.g., `agentpack policy lock`).
Retryable: yes.
Recommended action: create `repo/agentpack.org.yaml` (governance is opt-in) and retry.
Details: includes `{path, hint}`.

### E_POLICY_CONFIG_INVALID
Meaning: `repo/agentpack.org.yaml` is syntactically or semantically invalid (e.g., invalid YAML, missing/empty `policy_pack.source`, unsupported `policy_pack.source` syntax).
Retryable: depends on fixing config.
Recommended action: fix YAML based on `details` and retry.
Details: includes `{path, error?}` and MAY include `{field, value, hint}`.

### E_POLICY_CONFIG_UNSUPPORTED_VERSION
Meaning: `repo/agentpack.org.yaml` `version` is unsupported.
Retryable: depends on upgrading agentpack or fixing config.
Recommended action: set `version` to a supported value (currently `1`) or upgrade agentpack.
Details: includes `{path, version, supported}`.

## Non-stable / fallback error codes

### E_UNEXPECTED
Meaning: unexpected failure that was not classified as a stable UserError.
Retryable: unknown.
Recommended action:
- Save `errors[0].message` plus surrounding context (stdout/stderr).
- Retry with a smaller repro.
- For automation: typically “escalate to human” or fail-fast, rather than branching on message text.
