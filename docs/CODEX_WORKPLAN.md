# CODEX_WORKPLAN.md

Agentpack engineering + OSS workplan (designed to be executable by Codex CLI / automation)

> Current as of **v0.5.0** (2026-01-13). v0.5.0 delivered a round of correctness hardening (fs_key prefix length cap, end-to-end atomic writes, adopt protection, sparse overlays + rebase, CLI split, overlay metadata + doctor checks, etc.). This workplan focuses on the next steps to make the project “truly great” as open source.

Recommended usage:
- One task per PR (or per commit group). PR description should include: intent, acceptance criteria, and regression test commands.
- Each PR should run at least: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

------------------------------------------------------------
P0: regression tests and contract lock-down (highest priority)
------------------------------------------------------------

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
- Coverage (see `TARGET_CONFORMANCE.md`):
  - delete protection (delete managed only)
  - manifests (per-root `.agentpack.manifest.json`)
  - drift (missing/modified/extra)
  - rollback (restorable)

P0-3 Windows path and permission cases
- Goal: ensure overlay/deploy won’t be easily broken by Windows path characters/length/permissions.
- Suggestion:
  - Unit tests already cover `module_fs_key` truncation/stability; add integration coverage that overlay edit/rebase output paths are usable on Windows runners.

------------------------------------------------------------
P1: product UX (daily-usable)
------------------------------------------------------------

P1-1 Better status output (without breaking JSON)
- Human mode: group by root and provide “next action” suggestions (e.g., “run bootstrap” / “run deploy --apply”).
- JSON mode: additive fields like `summary` are OK; do not delete/rename existing fields.

P1-2 Better explainability for evolve propose
- Make skipped reasons more structured and actionable:
  - missing → suggest `evolve restore` or `deploy --apply`
  - multi_module_output → suggest adding markers or splitting outputs

P1-3 Docs/examples (more user-friendly)
- Docs are consolidated; next suggested additions:
  - A minimal example repo (with `agentpack.yaml` + a few modules)
  - A “0 → multi-machine sync” screencast/GIF (optional)

------------------------------------------------------------
OSS: open-source project ops (GitHub settings are handled separately)
------------------------------------------------------------

OSS-1 Contributor experience
- `CONTRIBUTING.md` (root) + issue/PR templates
- `CODE_OF_CONDUCT.md`
- `SECURITY.md` (vulnerability reporting)
- `LICENSE`

OSS-2 Release and distribution
- Ensure the `cargo-dist` release workflow is stable across three platforms (artifacts, checksums, optional signing).

For GitHub-side manual setup items, see `GITHUB_SETUP.md`.
