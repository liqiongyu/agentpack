---
status: active
owner: liqiongyu
last_updated: 2026-01-19
superseded_by: ""
scope: governance
---

# Codex execution guide (Agentpack repo)

This is the single active execution guide for Codex (and other coding agents) working on Agentpack.

## Ground rules

- Keep PRs small: one backlog item per PR, ideally ≤ 200–500 effective LOC (excluding docs/tests when reasonable).
- Do not break userspace:
  - `--json` is a stable contract (`schema_version=1`): additive-only changes.
  - Stable error codes MUST remain stable; new codes require docs + tests.
- Prefer spec-driven work for behavior changes:
  - Backlog/spec → OpenSpec change → issue → PR → review → fix → merge → archive OpenSpec change.

## Where the contracts live

- CLI behavior and stable contracts: `docs/SPEC.md`
- JSON envelope contract: `docs/reference/json-api.md`
- Error codes: `docs/reference/error-codes.md`
- Roadmap / execution backlog: `docs/dev/roadmap.md`
- OpenSpec capabilities + requirements: `openspec/specs/`

## Standard workflow (per backlog item)

1. Read the relevant specs and backlog entry.
2. Create an OpenSpec change (`openspec new change <id>`) and write:
   - `proposal.md`, `tasks.md`, and any spec deltas under `openspec/changes/<id>/specs/`
3. Validate: `openspec validate <id> --strict`
4. Create a GitHub issue describing scope + acceptance criteria.
5. Implement on a branch; keep changes focused.
6. Run checks locally (at minimum):
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all --locked`
7. Open a PR referencing the issue and OpenSpec change.
8. Review the PR using the repo’s review checklist (and comment the review).
9. Merge once CI is green.
10. Archive the OpenSpec change in a separate PR:
    - `openspec archive <id> --yes`

## Tests and docs expectations

- Any change that affects stable output (especially `--json`) MUST include tests (golden/contract).
- Any change that affects user-facing behavior MUST update user docs under `docs/` and keep doc-sync tests passing.
- Avoid network in tests; use temp dirs and isolated env vars (e.g., `AGENTPACK_HOME`).
