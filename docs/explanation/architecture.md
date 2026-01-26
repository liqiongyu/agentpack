# Architecture

> Language: English | [Chinese (Simplified)](../zh-CN/explanation/architecture.md)

> Current as of **v0.8.0** (2026-01-20). Historical content is tracked in git history.

## 1. One-line summary

Agentpack = “a declarative asset compiler + a safe applier”.

Inputs:
- manifest (what you want: modules/profiles/targets)
- overlays (how you customize: global/machine/project layers)
- lockfile (which versions: git sources locked to commit + sha256)

Outputs:
- target-discoverable directories/files (e.g. `~/.codex/skills/...`, `~/.claude/commands/...`)
- per-root `.agentpack.manifest.<target>.json` (safe deletes + drift/status)
- state snapshots (deploy/bootstrap/rollback snapshots)

## 2. Architecture diagram (high level)

```mermaid
flowchart TD
  M[agentpack.yaml<br/>manifest] --> C[Compose & materialize<br/>(per module)]
  L[agentpack.lock.json<br/>lockfile] --> C
  O[overlays<br/>(global / machine / project)] --> C

  C --> R[Render desired state<br/>(per target)]
  R --> P[Plan / Diff]
  P -->|dry run| OUT[Human output / JSON envelope]
  P -->|deploy --apply| A[Apply (writes)]

  A --> MF[Write target manifest<br/>.agentpack.manifest.&lt;target&gt;.json]
  A --> SS[Create snapshot<br/>state/snapshots/]
  A --> EV[Record events<br/>state/logs/]

  SS --> RB[Rollback]
```

## 3. Three-layer storage model (separate by design)

A) Config repo (git-managed, syncable)
- `agentpack.yaml`
- `modules/` (optional: in-repo modules)
- `overlays/` and `projects/` (customization and feedback-loop changes)

B) Cache/store (not in git)
- checkouts for git sources
- goal: reproducible, not necessarily auditable

C) Deployed outputs (not in git)
- final outputs written into target tool directories
- goal: always rebuildable; rollback uses snapshots

## 4. Key directories

Default `AGENTPACK_HOME=~/.agentpack` (overridable):
- `repo/`: config repo
- `cache/`: git sources cache
- `state/snapshots/`: deploy/rollback snapshots
- `state/logs/`: events.jsonl (record/score)

## 5. Core pipeline (engine)

1) Load
- Read `agentpack.yaml`
- Read/use lockfile (if present) to resolve git sources reproducibly
- Derive project identity (for project overlays) and machine id (for machine overlays)

2) Materialize (per module)
- Resolve upstream module root (local_path or git checkout under store)
- Compose in order: upstream → global → machine → project
- Validate module structure:
  - instructions must contain `AGENTS.md`
  - skill must contain `SKILL.md`
  - prompt/command must contain exactly one `.md` file
  - if a command uses bash, frontmatter must allow `Bash(...)`

3) Render (per target)
- codex: render skills/prompts/`AGENTS.md`
  - multiple instructions are combined into one `AGENTS.md` with section markers to support `evolve propose`
- claude_code: render commands (`~/.claude/commands/*.md` or `<repo>/.claude/commands/*.md`)

4) Plan / Diff
- Compute create/update/delete
- Classify `update_kind`:
  - managed_update: updating a managed file
  - adopt_update: overwriting an existing unmanaged file (refused by default; requires `--adopt`)

5) Apply
- Backup before writing
- Refresh target manifests (`.agentpack.manifest.<target>.json`) after writing
- Record a snapshot (for rollback)

## 6. Overlays

- Directory naming uses `module_fs_key = sanitize(prefix) + "--" + hash10` to avoid Windows invalid characters and overly long paths.
- Each overlay directory contains `.agentpack/` metadata:
  - `baseline.json`: upstream fingerprint (drift warnings + 3-way merge)
  - `module_id`: the original module id
- `overlay rebase` uses the baseline for 3-way merge and can `--sparsify` files identical to upstream.

## 7. JSON output and safety guardrails

- `--json` output is a stable envelope (`schema_version=1`) and remains valid JSON even on failures.
- To prevent accidental writes by scripts/LLMs, mutating commands in `--json` mode require explicit `--yes`.
- See `ERROR_CODES.md` for stable error codes.

## 8. Extensibility: Target SDK

Goal: make adding new targets predictable, reviewable, and testable.

- Code: `TargetAdapter` trait (`render(...)`)
- Docs: define roots, mapping rules, and validation rules
- Tests: conformance (delete protection, manifests, drift, rollback, JSON contract)

See: `TARGET_SDK.md` and `TARGET_CONFORMANCE.md`.
