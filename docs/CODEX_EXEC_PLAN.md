# CODEX_EXEC_PLAN.md

Agentpack “AI-first” execution roadmap (task-shaped, designed to be executable by Codex CLI / automation)

> Current as of **v0.5.0** (2026-01-13). This file consolidates the discussion around:
> - how agents (Codex / Claude Code) should use Agentpack
> - how Agentpack should evolve next (UX, integrations, overlays, targets, testing)
> - concrete engineering tasks (P0/P1/P2), each intended to fit in a single PR

Recommended usage:
- One task per PR (or per commit group).
- PR description should include: intent, acceptance criteria, and regression test commands.
- Every PR should run at least: `cargo fmt`, `cargo clippy`, `cargo test`.

------------------------------------------------------------
0) Mental model (what “AI-first” means in practice)
------------------------------------------------------------

AI-first is not “the model magically knows”. It is:

1) A stable machine interface
- The CLI’s `--json` envelope (`schema_version=1`) + stable error codes.
- Automation must be able to parse outputs, branch on errors, and retry safely.

2) A safe default workflow
- Agents should follow a state machine: `doctor → (update) → preview/plan → (diff) → deploy --apply (explicit approval) → status → (explain/evolve)`.

3) Operator assets that teach the workflow
- Codex: a Skill directory with `SKILL.md`.
- Claude Code: a set of `/ap-*` slash commands (and optionally an Agent Skill).
- Agentpack should ship/update these via `agentpack bootstrap`.

------------------------------------------------------------
P0: contract lock-down + conformance (highest priority)
------------------------------------------------------------

P0 items make later work safe. Do these before changing outputs or adding new targets.

P0-1 JSON contract golden tests
- Goal: lock down `schema_version=1` envelope fields + key error codes to prevent accidental breaking changes.
- Scope: integration tests that run the real CLI and snapshot JSON.
- Suggested approach:
  - Use temp dirs as a fake `AGENTPACK_HOME` and fake target roots.
  - Snapshot at least: `doctor`, `preview`, `diff`, `deploy --apply`, `status`, `rollback`.
  - Cover error scenarios:
    - `E_CONFIRM_REQUIRED` (`--json` mutating command without `--yes`)
    - `E_ADOPT_CONFIRM_REQUIRED` (adopt protection)
    - `E_DESIRED_STATE_CONFLICT` (conflicts)
    - `E_OVERLAY_REBASE_CONFLICT` (merge conflicts)
- Acceptance criteria:
  - Tests are deterministic (no network).
  - Snapshots are stable across machines.
  - Failing tests clearly identify contract regressions.

P0-2 Target conformance harness
- Goal: any new target must pass semantic consistency tests.
- Coverage (see `docs/TARGET_CONFORMANCE.md`):
  - delete protection (delete managed only)
  - manifests (`.agentpack.manifest.json` per root)
  - drift detection (missing/modified/extra)
  - rollback (restorable)
- Acceptance criteria:
  - Harness is runnable locally in CI-like mode.
  - At least two existing targets are validated by the harness.

P0-3 Windows path + permission edge cases
- Goal: overlays/deploy don’t break on Windows path rules.
- Scope:
  - Add integration coverage for overlay edit/rebase path construction.
  - Validate module_fs_key truncation + stability.
- Acceptance criteria:
  - Tests run on Windows CI runner (or at least guard the tricky cases with unit tests).

------------------------------------------------------------
P1: daily-usable product UX + operator assets
------------------------------------------------------------

P1 items improve day-to-day usability for humans and for agent automation.

P1-1 Better status output (without breaking JSON)
- Goal: make `status` actionable.
- Human mode:
  - Group drift by root.
  - Provide “next action” suggestions (e.g., “run bootstrap”, “run deploy --apply”, “run evolve propose”).
- JSON mode:
  - Additive fields only (e.g., `summary`, `next_actions`).
  - Do not rename/remove existing fields.
- Acceptance criteria:
  - Existing JSON parsers remain valid.
  - Human output becomes substantially more scannable.

P1-2 Better explainability for evolve propose
- Goal: make skipped reasons structured and actionable.
- Changes:
  - Expand the `skipped` payload to include:
    - `reason` (stable string enum)
    - `suggested_next_actions` (command strings)
    - optional `details` (file list / attribution hints)
  - Suggested mappings:
    - `missing` → suggest `evolve restore` or `deploy --apply`
    - `multi_module_output` → suggest adding markers or splitting outputs
- Acceptance criteria:
  - Output remains stable in `--json` (additive only).
  - The agent can decide what to do next without human interpretation.

P1-3 Operator assets improvements (Codex + Claude Code)

P1-3a Codex skill format compliance
- Goal: ensure shipped Codex skills follow the current Codex skills format.
- Changes:
  - Ensure each `templates/codex/skills/*/SKILL.md` starts with YAML frontmatter containing `name` and `description`.
  - Keep extra keys (e.g., `agentpack_version`) in frontmatter (Codex ignores unknown keys).
- Acceptance criteria:
  - Codex loads the skill without validation errors.
  - Bootstrap writes the updated template into `~/.codex/skills/...`.

P1-3b Claude Code “Agent Skill” (optional complement to /ap-*)
- Goal: allow Claude to more naturally discover “I should use agentpack now” while keeping explicit /ap-* buttons.
- Proposed design:
  - Add a Claude Agent Skill directory describing when to use Agentpack.
  - Keep `/ap-*` commands as explicit execution wrappers with minimal `allowed-tools`.
- Acceptance criteria:
  - Skill is safe-by-default (mutating operations still require explicit user request).
  - No duplication: the Skill references `/ap-*` commands for execution.

P1-3c Mutating command safety (Claude Code)
- Goal: avoid accidental programmatic invocation of mutating operations.
- Proposed design options:
  - Add `disable-model-invocation: true` to mutating slash commands (e.g., `/ap-deploy`, `/ap-update`) if we want to require explicit user invocation.
  - Alternatively, keep programmatic invocation allowed, but add stricter “user confirmation required” language and rely on Agentpack guardrails (`--yes` required in `--json`).
- Acceptance criteria:
  - Document the choice in `docs/BOOTSTRAP.md`.

P1-4 Project self-skill (this repo)
- Goal: make contributing to Agentpack self-serve for Codex.
- Changes:
  - Add `.codex/skills/agentpack-dev/` (this folder) to the repo.
  - Keep the skill as a thin entrypoint; detailed tasks live in this roadmap and `docs/CODEX_EXEC_PLAN.md`.
- Acceptance criteria:
  - Codex can discover the skill from repo scope.
  - The skill points to canonical specs and workplans.

------------------------------------------------------------
P2: deeper features (overlays, UI, ecosystem)
------------------------------------------------------------

P2-1 Patch-based overlays (optional)
- Goal: reduce churn for “edit one line” overlay use cases and make conflicts more readable.
- Constraints (recommended for v1):
  - Text files only (UTF-8), size capped.
  - Binary files: fall back to full-file overlay.
  - Patch format: line-based diff; store alongside baseline metadata.
- Semantics:
  - Desired state materialization applies patch on top of upstream (and lower-precedence overlays).
  - Rebase must remain 3-way merge aware (base = baseline.json).
- Acceptance criteria:
  - Deterministic patch apply; conflicts reported as `E_OVERLAY_REBASE_CONFLICT` (or a new stable code if needed).
  - Clear diagnostics listing conflict hunks/files.
  - Conformance tests cover patch overlays.

P2-2 Lightweight TUI (optional)
- Goal: read-only “browser” for plan/diff/status/snapshots.
- Design guardrails:
  - Must not replace the CLI.
  - Must reuse the same internal renderers/JSON as CLI.
  - Keep dependencies minimal.
- Acceptance criteria:
  - Works without network.
  - Provides a faster human workflow for inspection.

P2-3 TargetAdapter modularization (compile-time features)
- Goal: make it easy to add/ship targets without bloating the core.
- Approach:
  - Split targets behind Cargo features.
  - Provide a minimal “Target SDK” and conformance harness gating.
- Acceptance criteria:
  - Core build stays lean.
  - Adding a new target requires conformance tests.

P2-4 MCP server for Agentpack (future-facing)
- Goal: let agents call Agentpack as a structured tool (not just bash + JSON parsing).
- Proposed shape:
  - `agentpack-mcp` exposes `doctor/preview/diff/deploy/status/rollback/evolve` as tool calls.
  - Return the same JSON envelope fields (or embed them) so existing semantics stay aligned.
- Acceptance criteria:
  - Tool calls are deterministic and safe (mutating calls still require explicit approval semantics).
  - Documentation: how to wire into Codex MCP config.

------------------------------------------------------------
Appendix: “agent workflow” reference
------------------------------------------------------------

Safe default sequence for an agent operating Agentpack:

1) `agentpack doctor --json`
2) (Optional) `agentpack update --yes --json`
3) `agentpack preview --json` (or `plan --json`)
4) (Optional) `agentpack preview --diff --json` (or `diff --json`)
5) (Only with explicit approval) `agentpack deploy --apply --yes --json`
6) `agentpack status --json`
7) (Optional) `agentpack explain status --json`
8) (Optional) `agentpack evolve propose --yes --json` (creates proposal branch)
