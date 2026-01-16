# Daily workflows

> Language: English | [Chinese (Simplified)](zh-CN/WORKFLOWS.md)

This document provides a set of “daily driver” workflows. You can treat it as best practices, or hand it to Codex/Claude and have the agent execute the steps.

## 0) First-time adoption: import existing assets

If you already have existing assets (e.g., `~/.codex/prompts`, `~/.codex/skills`, `.claude/commands`, `AGENTS.md`) and want to bring them under agentpack management:

1. Initialize the config repo (once):
- `agentpack init --git`

2. Generate an import plan (dry-run):
- `agentpack import`
- Automation-friendly: `agentpack import --json`

3. Apply the import (writes into the config repo only):
- `agentpack import --apply`
- Automation-friendly: `agentpack import --apply --yes --json`

Notes:
- If `import` creates a project-scoped profile (e.g., `project-<project_id>`), use it for preview/apply in that project:
  - `agentpack --profile project-<project_id> preview --diff`
  - `agentpack --profile project-<project_id> deploy --apply`

## 1) The most common loop: update → preview → apply

1. Update dependencies (runs `lock` automatically if the lockfile is missing):
- `agentpack update`

2. Preview (recommended to always include diffs):
- `agentpack preview --diff`

3. Apply:
- `agentpack deploy --apply`

Common variants:
- Only for a profile: `agentpack --profile work update && agentpack --profile work preview --diff && agentpack --profile work deploy --apply`
- Only for a target: `agentpack --target codex preview --diff`

## 2) Multi-machine sync (treat the config repo as the single source of truth)

Recommended approach: manage the config repo with git and sync via rebase.

1. Set a remote:
- `agentpack remote set <url>`

2. Sync:
- `agentpack sync --rebase`

Conflicts:
- Agentpack does not resolve git merge conflicts for you. If a conflict happens, the command fails and you should resolve it with git manually.

## 3) Overwrite protection (`adopt_update`) and safe take-over

When your plan includes `adopt_update`:
- The destination path already exists, but it is not managed (i.e., it was not previously written/managed by agentpack).
- `deploy --apply` refuses to overwrite by default.

If you explicitly want to take over (overwrite and start managing the file):
- `agentpack deploy --apply --adopt`

Recommended:
- Run `agentpack preview --diff` first and understand the impact.
- Prefer overlays when possible to avoid overwriting hand-edited user files.

## 4) Local customization with overlays (prefer sparse overlays)

Typical scenario: you add an upstream skill/command, and want to tweak behavior or add a bit of local guidance.

1. Create a sparse overlay:
- `agentpack overlay edit <module_id> --sparse`

2. Keep only your changed files in the overlay directory (minimal diffs).

3. Preview / deploy again:
- `agentpack preview --diff`
- `agentpack deploy --apply`

After upstream updates (after you run `update` and get new commits):
- `agentpack overlay rebase <module_id> --sparsify`

If rebase conflicts:
- The command returns `E_OVERLAY_REBASE_CONFLICT` with the conflict file list.
- Open the conflicted files under the overlay directory, resolve, then re-run rebase (or commit the overlay changes directly).

## 5) Drift (status) → proposal (evolve propose) → review → merge

Goal: turn “changes on disk under target roots” into reviewable overlay changes in your config repo.

1. Check drift:
- `agentpack status`

2. Generate proposal candidates (recommended to start with dry-run):
- `agentpack evolve propose --dry-run --json`

3. Create a proposal branch (creates a git branch in the config repo and writes overlay files):
- `agentpack evolve propose --scope global`

4. Review in the config repo:
- `cd ~/.agentpack/repo && git status && git diff`
- Commit and open a PR (if you sync to a remote) or merge locally.

5. Deploy again to bring target roots back to desired state:
- `agentpack deploy --apply`

Notes:
- By default, proposals are generated only for drift that can be safely mapped back to a single module.
- For aggregated outputs (e.g., Codex `AGENTS.md`), if section markers are present, agentpack can map drift back to specific instructions module sections.

## 6) Restore missing files (evolve restore)

If `status` shows some managed files were deleted (missing) and you only want to recreate them (without updating/deleting anything else):

- Preview: `agentpack evolve restore --dry-run --json`
- Apply: `agentpack evolve restore`

Property:
- create-only: creates missing files only; does not update existing files and does not delete anything.

## 7) Automation / agent integration tips (`--json`)

If you treat agentpack as a programmable component (e.g., called by Codex CLI):
- Use `--json` output for decisions.
- On `E_CONFIRM_REQUIRED`: confirm the intent to mutate and retry with `--yes`.
- On `E_ADOPT_CONFIRM_REQUIRED`: retry with explicit `--adopt` to overwrite unmanaged existing files.

Suggested pattern:
- For mutating commands (deploy --apply / update / lock / fetch / bootstrap / evolve propose/restore / overlay edit/rebase / rollback, etc.), always pass `--yes` in automation, and run `preview` first.

See: `JSON_API.md`, `ERROR_CODES.md`.
