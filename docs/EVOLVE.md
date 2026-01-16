# Evolve (closed loop)

> Language: English | [Chinese (Simplified)](zh-CN/EVOLVE.md)

Evolve is not “auto-modify your system”. Its job is to turn changes into **reviewable, rollbackable, syncable** updates:
- Changes happen under target roots (e.g., you manually edit `~/.claude/commands/ap-plan.md`).
- Agentpack captures that drift into overlays (on a new branch in your config repo) so you can review and merge like normal code review.

Related commands:
- `agentpack record` / `agentpack score`
- `agentpack explain plan|diff|status`
- `agentpack evolve propose` / `agentpack evolve restore`

## 1) record / score (observability)

### record

`agentpack record` reads a JSON object from stdin and appends it to `state/logs/events.jsonl`.

Example:
- `echo '{"module_id":"command:ap-plan","success":true}' | agentpack record`

Notes:
- You can place `module_id`/`success` at the top-level or inside a nested event object.
- `score` tolerates malformed lines (e.g., truncated JSON), skips them, and emits warnings.

### score

`agentpack score` aggregates events.jsonl into simple stats (e.g., failure rates), used to:
- Find modules that fail often and should be fixed
- Provide a prioritization signal for evolve (extensible)

## 2) explain

`agentpack explain plan|diff|status` explains:
- Which module id produced a given output file
- Which source layer it came from (upstream/global/machine/project overlay)

This is critical when debugging “why is this version active?”

## 3) evolve propose (turn drift into overlays)

Command:
- `agentpack evolve propose [--module-id <id>] [--scope global|machine|project] [--branch <name>]`

Recommended flow:
1) Inspect candidates (no writes):
- `agentpack evolve propose --dry-run --json`

2) Create a proposal branch:
- `agentpack evolve propose --scope global`

Behavior and constraints:
- This is a mutating command: in `--json` mode you must pass `--yes`.
- Requires the config repo to be a git repo, and the working tree must be clean (otherwise it refuses).
- Creates a branch (default `evolve/propose-<scope>-<module>-<timestamp>`), writes drifted files into the appropriate overlay paths, then runs `git add -A` and attempts to commit.
  - If commit fails (e.g., missing git identity), changes are not lost: the branch and working tree changes remain.

### What drift is proposable?

The default strategy is conservative: only propose drift that can be safely mapped back to its source.

1) **Single-module output** (recommended):
- The output file has `module_ids.len() == 1`
- The file exists and content differs (`modified`)

2) **Aggregated output (combined instructions files)**
- When multiple instructions modules are combined into a single instructions output (e.g. Codex `AGENTS.md`, VS Code `.github/copilot-instructions.md`), agentpack wraps each module section with markers:

```md
<!-- agentpack:module=instructions:one -->
...content...
<!-- /agentpack -->
```

- If both deployed and desired contain markers, evolve propose can diff sections per module and write changes back to that module’s overlay.

These cases are skipped (reported in `skipped` with a reason):
- `missing`: file does not exist (see evolve restore)
- `multi_module_output`: cannot safely attribute to a single module
- `read_error`: failed to read the file

## 4) evolve restore (restore missing files; create-only)

Command:
- `agentpack evolve restore [--module-id <id>]`

Use case:
- Some managed files were deleted (missing) and you only want to recreate them without updating/deleting anything else.

Properties:
- create-only: creates missing files only
- does not update existing files
- does not delete anything

Recommended:
- Run `--dry-run --json` first to inspect which paths would be restored.

## 5) Relationship to overlays

- The output of evolve propose is overlays.
- The intended flow is: review/merge into the config repo, then run `deploy --apply` to bring target roots back to desired state.

If you want to do it manually:
- You can run `agentpack overlay edit --sparse <module_id>` and copy drifted content into the overlay yourself.
