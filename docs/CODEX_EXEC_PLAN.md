# CODEX_EXEC_PLAN.md

Agentpack spec reconciliation + backlog + next workplan (designed to be executable by Codex CLI / automation)

> Current as of **v0.5.0** (2026-01-13). This document consolidates:
> - the current spec-alignment drift report across `docs/SPEC.md`, `docs/JSON_API.md`, `docs/ERROR_CODES.md`, and `openspec/specs/`
> - the v0.5.0 backlog snapshot (`docs/BACKLOG.md`)
> - an integrated P0/P1/P2 execution plan (including additional hardening items surfaced during review)

This file is intentionally “task-shaped”: one task per PR is the default expectation.

Recommended usage:
- One task per PR (or per commit group). PR description should include: intent, acceptance criteria, and regression test commands.
- Each PR should run at least: `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`.
- If you touch OpenSpec specs or any stable contract, also run: `openspec validate --all --strict --no-interactive`.

------------------------------------------------------------
0) Canonical sources of truth (do not improvise)
------------------------------------------------------------

Agentpack’s spec system is layered and must stay consistent:

1) `docs/SPEC.md`
- The implementation-level contract and the project’s single authoritative spec.
- If anything conflicts, reconcile toward `docs/SPEC.md` unless the implementation/CI clearly proves `docs/SPEC.md` is stale.

2) `docs/JSON_API.md` + `docs/ERROR_CODES.md`
- The stable machine contract for `--json`.
- Compatibility policy for `schema_version=1`: additive only (no rename/remove), semantics must not change without bumping schema_version.

3) `openspec/specs/`
- The OpenSpec “requirements slice” used for proposal-driven work.
- It MUST remain consistent with `docs/SPEC.md`. If it drifts, reconcile promptly.

4) `openspec/changes/archive/`
- Historical, completed change records.
- It is not expected to match current behavior verbatim; it exists for traceability. However, archived requirements should remain discoverable in current `openspec/specs/` unless explicitly superseded.

------------------------------------------------------------
1) How to update documentation during development (the house rule)
------------------------------------------------------------

Before coding, classify your change:

A) Contract / externally observable change
Examples:
- changes to `--json` envelope fields, error codes, or payload semantics
- CLI behavior changes that automation depends on
- file format schema changes (manifest/lockfile/snapshots)
- safety rules (adopt protection, delete protection, overlay precedence, rollback semantics)

Process:
1. Update `docs/SPEC.md` (semantic truth) and the relevant contract docs (`docs/JSON_API.md`, `docs/ERROR_CODES.md`) if applicable.
2. Update OpenSpec requirements:
   - If this is a new capability or meaningful contract shift: create a change under `openspec/changes/<change-id>/` with proposal/tasks/delta specs.
   - Run `openspec validate <change-id> --strict --no-interactive` before implementation.
   - After merge: archive via `openspec archive <change-id> --yes` in a separate PR.
3. Add/adjust regression tests:
   - If stable `--json` output may change: add/update golden snapshots under `tests/golden/` or CLI integration tests.
4. Keep compatibility:
   - For `schema_version=1`, only additive JSON field changes are allowed.

B) Internal refactor / engineering hardening (no contract change)
Examples:
- better error messages without code changes
- internal dedup/perf improvements
- reliability fallbacks that do not alter stable outputs

Process:
- Update `docs/CODEX_WORKPLAN.md` (task list) and/or `docs/BACKLOG.md` (product backlog) as appropriate.
- Update tests if behavior changes in a way that could regress.

------------------------------------------------------------
2) Spec alignment drift report (as of v0.5.0)
------------------------------------------------------------

This section lists concrete, known drifts that should be reconciled. The primary goal is consistency between:
- `docs/SPEC.md` (authoritative implementation contract)
- `docs/JSON_API.md` (stable JSON contract)
- `openspec/specs/*/spec.md` (requirements slice)

2.1 Drift: JSON envelope field list in OpenSpec CLI spec
- `openspec/specs/agentpack-cli/spec.md` “JSON output contract” lists top-level fields as:
  `ok`, `command`, `version`, `data`, `warnings`, `errors`
- `docs/SPEC.md` and `docs/JSON_API.md` require `schema_version` as part of every `--json` envelope.

Reconcile:
- Update OpenSpec CLI spec to include `schema_version` as a required top-level field.
- Ensure wording matches `docs/JSON_API.md` envelope shape and compatibility policy.

2.2 Drift: Overlay precedence missing machine layer in OpenSpec CLI spec
- `docs/SPEC.md` defines overlay precedence as:
  upstream → global → machine → project (with global `--machine`)
- `openspec/specs/agentpack-cli/spec.md` still states:
  upstream → global → project

Reconcile:
- Update OpenSpec CLI spec overlay precedence requirement and scenarios to include the machine layer and `--machine`.

2.3 Drift: Overlay path placeholders / examples inconsistent with module_fs_key
- `docs/SPEC.md` defines on-disk overlay directories using `<module_fs_key>`.
- `openspec/specs/agentpack/spec.md` overlay editing paths still use `<moduleId>` / `<projectId>` in examples, while the same spec also requires module_fs_key for disk addressing.

Reconcile:
- Normalize placeholder conventions:
  - `<module_id>` = user-facing identifier passed to commands
  - `<module_fs_key>` = filesystem-safe directory key derived from module_id, used on disk
  - `<project_id>` and `<machine_id>` remain identity values
- Update OpenSpec overlay path examples to reflect on-disk paths correctly.

2.4 Drift: Drift-kind naming differs between SPEC/OpenSpec and JSON contract
- `docs/SPEC.md` and `openspec/specs/agentpack/spec.md` use “changed/missing/extra” in manifest-based drift language.
- `docs/JSON_API.md` defines `status` JSON drift kind as `missing|modified|extra`.
- `docs/CLI.md` also describes drift as `missing/modified/extra`.

Reconcile (recommended):
- Treat `modified` as the canonical JSON kind (stable API).
- In `docs/SPEC.md` and OpenSpec core spec, either:
  - change wording to “modified/missing/extra”, or
  - explicitly clarify: human output may say “changed”, but JSON `kind` uses `modified`.

2.5 Drift: help/schema command coverage boundary
- OpenSpec CLI spec includes `agentpack help --json` and `agentpack schema` as part of the CLI contract.
- `docs/CLI.md` and `docs/JSON_API.md` mention these commands, but `docs/SPEC.md` CLI chapter does not clearly cover them.

Reconcile (recommended):
- Add a short “Utility commands” subsection in `docs/SPEC.md` documenting:
  - `help --json` minimal requirements (commands list + mutating_commands)
  - `schema` minimal requirements (envelope + key payload shapes)
- Cross-link to `docs/JSON_API.md` for full shape.

2.6 Optional consistency choice: E_UNEXPECTED mention in SPEC.md
- `docs/JSON_API.md` and `docs/ERROR_CODES.md` document `E_UNEXPECTED` as the non-stable fallback.
- `docs/SPEC.md` lists stable codes and references `ERROR_CODES.md`, but does not mention the fallback.

Reconcile (optional):
- Keep `E_UNEXPECTED` out of the stable list (it is non-stable), but add a one-line note in `docs/SPEC.md` pointing to the fallback definition.

------------------------------------------------------------
3) Spec reconciliation tasks (P0, one PR each)
------------------------------------------------------------

These tasks are documentation-only unless otherwise noted. They should be done early to reduce future churn.

P0-SPEC-1 Fix JSON envelope field list in OpenSpec CLI spec
- Goal: align OpenSpec CLI contract with `docs/SPEC.md` + `docs/JSON_API.md` envelope.
- Changes:
  - Update `openspec/specs/agentpack-cli/spec.md` JSON output contract requirement to include `schema_version`.
- Acceptance criteria:
  - `openspec validate --all --strict --no-interactive` passes.
  - No behavior change required (doc-only).
- Regression commands:
  - `openspec validate --all --strict --no-interactive`

P0-SPEC-2 Fix overlay precedence in OpenSpec CLI spec (add machine layer)
- Goal: align overlay precedence with `docs/SPEC.md` (upstream→global→machine→project).
- Changes:
  - Update `openspec/specs/agentpack-cli/spec.md` overlay precedence requirement and scenarios.
  - Ensure it references the `--machine` flag semantics (do not redefine; just align).
- Acceptance criteria:
  - `openspec validate --all --strict --no-interactive` passes.

P0-SPEC-3 Normalize overlay path placeholders to module_fs_key in OpenSpec core spec
- Goal: remove contradictory placeholders and align with module_fs_key disk-addressing rule.
- Changes:
  - Update `openspec/specs/agentpack/spec.md` overlay editing path examples:
    - Replace `<moduleId>` with `<module_fs_key>` where describing on-disk paths.
    - Keep `<module_id>` when describing CLI inputs.
  - Ensure examples match `docs/SPEC.md` overlay mapping.
- Acceptance criteria:
  - `openspec validate --all --strict --no-interactive` passes.

P0-SPEC-4 Resolve drift-kind naming (changed vs modified)
- Goal: make drift terminology unambiguous across SPEC/OpenSpec and JSON API.
- Recommended decision:
  - JSON kind remains `missing|modified|extra`.
  - Docs/spec text should either adopt `modified` or clearly map “changed” → `modified` in JSON.
- Changes:
  - Update `docs/SPEC.md` target manifest/status drift wording (and/or add an explicit mapping note).
  - Update `openspec/specs/agentpack/spec.md` manifest-based status requirement wording accordingly.
- Acceptance criteria:
  - Documentation is consistent: readers can predict JSON `kind` without guessing.
  - If any golden tests assert string output, update them intentionally.

P0-SPEC-5 Document help/schema commands in docs/SPEC.md (minimal contract)
- Goal: remove “contract boundary ambiguity” between SPEC vs CLI/JSON docs.
- Changes:
  - Add a brief `help --json` and `schema` subsection in `docs/SPEC.md` CLI chapter.
  - Cross-link to `docs/JSON_API.md` for shape details.
- Acceptance criteria:
  - `docs/SPEC.md` describes the existence and minimal requirements of these commands.
  - OpenSpec CLI spec and SPEC no longer disagree on whether these commands are “in contract”.

P0-SPEC-6 Add optional note about E_UNEXPECTED fallback in docs/SPEC.md
- Goal: reduce confusion without making fallback “stable”.
- Changes:
  - Add a short note near stable error code section referencing `docs/JSON_API.md` / `docs/ERROR_CODES.md` fallback behavior.
- Acceptance criteria:
  - Stable list remains stable-only; fallback is documented as non-stable.

------------------------------------------------------------
4) Backlog snapshot (verbatim from docs/BACKLOG.md, v0.5.0)
------------------------------------------------------------

> Current as of **v0.5.0** (2026-01-13). Historical content is tracked in git history.

Status
- v0.5 milestone: a round of “daily-usable + AI-first loop” convergence (composite commands, overlay rebase, adopt protection, evolve restore, etc.).
- For concrete shipped changes, see `CHANGELOG.md`.

Next (candidates for v0.6+)

Targets & ecosystem
- Add new targets (Cursor / VS Code, etc.), gated by: TargetAdapter + conformance tests.
- For each new target: mapping rules, examples, migration notes.

UX & ergonomics
- Stronger `status` output (optional summary, grouped by root, actionable suggestions).
- Richer but still script-friendly warnings (with actionable commands where possible).
- Consider a lightweight TUI (browse plan/diff/status/snapshots) while keeping the core usable in non-interactive mode.

Overlays & evolve
- Patch-based overlays (optional): make small text edits easier to merge and conflicts more readable.
- Expand evolve propose coverage: better attribution for multi-module aggregated outputs (beyond AGENTS.md) and more structured skipped reasons.
- Provide clearer “next command” suggestions in evolve output (good for operator assets).

Engineering
- CLI golden tests (regression coverage for JSON output/error codes).
- Stronger conformance harness (temp roots, cross-platform path cases).
- Keep docs consolidated (legacy `docs/versions/` removed; rely on git history for iteration tracking).

------------------------------------------------------------
5) Integrated workplan (P0/P1/P2)
------------------------------------------------------------

This section merges:
- existing `docs/CODEX_WORKPLAN.md`
- backlog “Next” items (above)
- additional engineering improvements surfaced during review

------------------------------------------------------------
P0: regression tests + contract lock-down + spec reconciliation (highest priority)
------------------------------------------------------------

P0-0 Spec reconciliation (the P0-SPEC-* tasks in section 3)
- Do these first to stop further drift from compounding.

P0-1 JSON contract golden tests (strongly recommended)
- Goal: lock down the `schema_version=1` envelope fields and key error codes (avoid accidental breaking changes).
- Suggested approach:
  - Use temp dirs as a “fake AGENTPACK_HOME” and “fake target roots”.
  - Run the real CLI (or call internal command entrypoints) and snapshot/golden `plan/preview(deep)/deploy --apply/status/rollback` plus key error scenarios.
  - At minimum cover:
    - `E_CONFIRM_REQUIRED` (`--json` without `--yes`)
    - `E_ADOPT_CONFIRM_REQUIRED` (`adopt_update`)
    - `E_DESIRED_STATE_CONFLICT` (conflicts)
    - `E_OVERLAY_REBASE_CONFLICT` (merge conflicts)

P0-2 Conformance harness
- Goal: before adding a new target, you must be able to run a suite of “semantic consistency tests”.
- Coverage:
  - delete protection (delete managed only)
  - manifests (per-root `.agentpack.manifest.json`)
  - drift (missing/modified/extra)
  - rollback (restorable)

P0-3 Windows path and permission cases
- Goal: ensure overlay/deploy won’t be easily broken by Windows path characters/length/permissions.
- Suggestion:
  - Unit tests already cover `module_fs_key` truncation/stability; add integration coverage that overlay edit/rebase output paths are usable on Windows runners.

------------------------------------------------------------
P1: product UX (daily-usable) + reliability hardening
------------------------------------------------------------

P1-1 Better status output (without breaking JSON)
- Human mode:
  - group by root
  - provide actionable “next action” suggestions (e.g. “run bootstrap” / “run deploy --apply”)
- JSON mode:
  - additive fields like `summary` are OK; do not delete/rename existing fields.
  - Optional: add `data.suggestions[]` as additive (commands + reasons) so operator assets can chain actions without string parsing.

Acceptance criteria:
- Existing JSON fields remain unchanged.
- New fields are additive only.
- Update docs (`docs/JSON_API.md`) to mention the new additive fields if implemented.

P1-2 Better explainability for evolve propose
- Make skipped reasons more structured and actionable:
  - missing → suggest `evolve restore` or `deploy --apply`
  - multi_module_output → suggest adding markers or splitting outputs
- If JSON output changes (additive only), update `docs/JSON_API.md`.

P1-3 Docs/examples (more user-friendly)
- A minimal example repo (with `agentpack.yaml` + a few modules)
- A “0 → multi-machine sync” screencast/GIF (optional)

P1-4 Target manifest forward-compat handling (upgrade robustness)
- Goal: reduce hard failures when encountering unknown future manifest versions.
- Recommended behavior:
  - Read path: for status/doctor, unknown `schema_version` in `.agentpack.manifest.json` should degrade gracefully (warn + treat as missing manifest) rather than hard-failing the command.
  - Write path: continue writing the current manifest version.
- This is a contract-adjacent change; update `docs/SPEC.md` and OpenSpec requirements if behavior changes.

P1-5 Git shallow clone fallback reliability
- Goal: reduce “occasionally cannot checkout pinned commit” issues when using shallow clone.
- Suggested behavior:
  - If shallow clone/checkout fails, retry once with a non-shallow fetch path (or emit a clear actionable error pointing to `shallow=false`).
- Keep outputs stable; do not introduce nondeterministic network behavior in tests (tests should mock or use fixtures).

P1-6 Init ergonomics (optional)
- Consider:
  - `agentpack init --git`: also run `git init` and write a minimal `.gitignore` (idempotent).
  - `agentpack init --bootstrap`: optionally install operator assets or print next-step commands.
- If implemented, treat as CLI contract change and update `docs/SPEC.md` + `docs/CLI.md` + OpenSpec.

------------------------------------------------------------
P2: ecosystem expansion + engineering polish
------------------------------------------------------------

P2-1 New targets (Cursor / VS Code, etc.)
- Gate: do not add targets until P0-2 conformance harness exists and passes.
- For each new target:
  - mapping rules
  - examples
  - migration notes
- Add adapter-level tests and conformance coverage for each new target.

P2-2 Patch-based overlays (optional)
- Goal: make small text edits easier to merge and conflicts more readable.
- Likely requires a proposal (OpenSpec change) and careful compatibility discussion.

P2-3 Lightweight TUI (optional)
- Goal: browse plan/diff/status/snapshots while keeping core usable non-interactively.
- Do not block the CLI-first experience; treat as optional UI layer.

P2-4 TargetAdapter pluginization (architecture)
- Goal: reduce friction for community-contributed targets (compile-time features or external adapter crates).
- Keep core deterministic and minimal; avoid runtime plugin loading unless the safety story is clear.

P2-5 Store dedup (disk efficiency)
- Goal: avoid duplicate checkouts when multiple modules share the same git url+commit.
- Approach:
  - key store directories by hash(url)+commit instead of module_id+commit
  - keep legacy behavior for existing paths; migration must be safe.

P2-6 Optional durability mode (fsync)
- Goal: allow stronger crash consistency when desired.
- Suggestion:
  - add an opt-in flag or env var (e.g. `AGENTPACK_FSYNC=1`) to `write_atomic` paths.
- Keep default behavior unchanged; ensure cross-platform compatibility.

P2-7 Lockfile version validation / evolution policy
- Goal: future-proof lockfile upgrades with clear errors and guidance.
- If implemented:
  - validate lockfile `version` on read
  - return actionable stable error code when unsupported (may require new code + docs update)
- This touches contract: update `docs/SPEC.md`, `docs/ERROR_CODES.md`, and OpenSpec if behavior changes.

------------------------------------------------------------
6) PR checklist (copy into every PR description)
------------------------------------------------------------

- [ ] Intent and acceptance criteria are clear (what should be true after merge).
- [ ] Ran: `cargo fmt --all -- --check`
- [ ] Ran: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Ran: `cargo test --all --locked`
- [ ] If spec/contract touched:
  - [ ] Updated `docs/SPEC.md` as needed
  - [ ] Updated `docs/JSON_API.md` / `docs/ERROR_CODES.md` as needed
  - [ ] Updated `openspec/specs/` and ran: `openspec validate --all --strict --no-interactive`
  - [ ] Updated golden snapshots under `tests/golden/` (if stable output changed)
- [ ] If a new mutating command was added/changed:
  - [ ] Updated mutating command registry and `help --json` output expectations
  - [ ] Added/updated guardrails tests (`--json` requires `--yes`)
