# ERROR_CODES.md (stable error code registry)

> Current as of **v0.9.0** (2026-01-23). `SPEC.md` is the semantic source of truth; this file is the stable registry for `--json` automation (`errors[0].code`).

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

### E_GIT_WORKTREE_DIRTY
Meaning: the command requires a clean config repo git working tree, but uncommitted changes were detected.
Typical cases: `sync`, `evolve propose`.
Retryable: yes.
Recommended action: commit or stash your changes, then retry.
Details: includes `{command, repo, repo_posix, hint}`.

### E_GIT_REPO_REQUIRED
Meaning: the command requires the config repo to be a git repository, but `.git` was not found.
Typical cases: `sync`, `evolve propose`.
Retryable: yes.
Recommended action: initialize git in the config repo (e.g. `agentpack init --git`), then retry.
Details: includes `{command, repo, repo_posix, hint}`.

### E_GIT_DETACHED_HEAD
Meaning: the command refused to run because the config repo is on a detached HEAD.
Typical cases: `sync`.
Retryable: yes.
Recommended action: check out a branch (not detached HEAD), then retry.
Details: includes `{command, repo, repo_posix, hint}`.

### E_GIT_REMOTE_MISSING
Meaning: the command requires a configured git remote in the config repo, but it was not found.
Typical cases: `sync`.
Retryable: yes.
Recommended action: set the remote via `agentpack remote set <url> --name <remote>` (or `git remote add <remote> <url>`), then retry.
Details: includes `{command, repo, repo_posix, remote, hint}`.

### E_GIT_NOT_FOUND
Meaning: `git` executable was not found (not installed or not on PATH), but the command requires git operations.
Typical cases: any command that shells out to `git` (e.g. `sync`, `evolve propose`, git-sourced module workflows).
Retryable: yes.
Recommended action: install git and ensure `git` is available on PATH, then retry.
Details: includes `{cwd, cwd_posix, args, hint}`.

### E_CONFIRM_TOKEN_REQUIRED
Meaning: in MCP mode (`agentpack mcp serve`), `deploy_apply` was called with `yes=true` but without a `confirm_token` from the `deploy` tool.
Retryable: yes.
Recommended action: call the `deploy` tool, obtain `data.confirm_token`, then retry `deploy_apply` with that token.

### E_CONFIRM_TOKEN_EXPIRED
Meaning: in MCP mode (`agentpack mcp serve`), the provided `confirm_token` is expired.
Retryable: yes.
Recommended action: call the `deploy` tool again to obtain a fresh token, then retry `deploy_apply`.

### E_CONFIRM_TOKEN_MISMATCH
Meaning: in MCP mode (`agentpack mcp serve`), the provided `confirm_token` does not match the current `deploy` plan.
Retryable: yes.
Recommended action: call the `deploy` tool again and ensure the subsequent `deploy_apply` uses the matching token (and the same repo/profile/target/machine inputs).

### E_TTY_REQUIRED
Meaning: the command requires a real TTY (stdin and stdout must be terminals), but the current context is non-interactive.
Typical cases: `init --guided --json` in CI or when stdout is redirected/piped.
Retryable: yes.
Recommended action: run the command in an interactive terminal (avoid redirecting stdout; ensure stdin is a terminal).
Details: includes `{stdin_is_terminal, stdout_is_terminal, hint}`.

### E_ADOPT_CONFIRM_REQUIRED
Meaning: `deploy --apply` would overwrite an existing unmanaged file (`adopt_update`), but `--adopt` was not provided.
Retryable: yes.
Recommended action:
- Run `preview --diff` to confirm scope/impact.
- If you truly want to take over and overwrite, retry with `--adopt`.
Details: includes `{flag, adopt_updates, sample_paths}`.
Details also includes additive refusal guidance fields: `{reason_code, next_actions}`.

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
- `--target` must be `all|codex|claude_code|cursor|vscode|jetbrains|zed` (but feature-gated builds may support a subset; see `agentpack help --json` `data.targets[]`).
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

### E_IO_PERMISSION_DENIED
Meaning: a filesystem write failed due to permissions (including read-only destination files) or an access-denied condition.
Retryable: maybe.
Recommended action:
- Ensure the destination path (and its parent directories) are writable.
- On Windows, ensure the destination is not locked by another process (e.g., an editor) and retry.
Details: includes `{path, path_posix, raw_os_error?, hint}`.

### E_IO_INVALID_PATH
Meaning: a filesystem write failed because the destination path is invalid for the current platform (e.g., invalid characters on Windows).
Retryable: no (until path is fixed).
Recommended action: fix the configured destination path (remove invalid characters / use a valid root) and retry.
Details: includes `{path, path_posix, raw_os_error?, hint}`.

### E_IO_PATH_TOO_LONG
Meaning: a filesystem write failed because the destination path exceeds platform limits.
Retryable: no (until path is shortened or platform configuration is changed).
Recommended action:
- Use a shorter workspace/home path, or
- On Windows, enable long path support if applicable.
Details: includes `{path, path_posix, raw_os_error?, hint}`.

## Non-stable / fallback error codes

### E_UNEXPECTED
Meaning: unexpected failure that was not classified as a stable UserError.
Retryable: unknown.
Recommended action:
- Save `errors[0].message` plus surrounding context (stdout/stderr).
- Retry with a smaller repro.
- For automation: typically “escalate to human” or fail-fast, rather than branching on message text.
