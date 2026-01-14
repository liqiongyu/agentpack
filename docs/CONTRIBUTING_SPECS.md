# Contributing: Specs (OpenSpec)

This repo uses **spec-driven development** for changes that affect stable contracts (CLI semantics, `--json` output, file formats, etc.).

If you’re not sure whether something is a “contract change”, default to OpenSpec: it’s cheaper than shipping an accidental breaking change.

## Canonical contracts (read first)

- `docs/SPEC.md` (authoritative implementation-level contract)
- `docs/JSON_API.md` + `docs/ERROR_CODES.md` (`--json` envelope + stable error codes)
- `docs/CODEX_EXEC_PLAN.md` (AI-first execution roadmap)
- `openspec/AGENTS.md` + `openspec/project.md` (OpenSpec workflow + conventions)

## When you MUST use OpenSpec

Use OpenSpec when your change impacts any stable/external contract, including:

- **CLI semantics** (new command, new flag, changed defaults, changed safety model).
- **`--json` output** (new fields are allowed/additive, but still need to be specified and tested).
- **Stable error codes** (`E_*` codes or the classification rules that map failures to codes).
- **User-facing file formats**:
  - `agentpack.yaml` (manifest)
  - `agentpack.lock.json` (lockfile)
  - overlay metadata under `.agentpack/`
  - `.agentpack.manifest.json` (managed file boundary)
- **Architecture shifts** that change observable behavior (performance/security work that changes outcomes).

## When you can skip OpenSpec

You can usually skip OpenSpec for:

- Bug fixes that restore existing spec behavior (no contract change).
- Typos / formatting / docs-only updates.
- Internal refactors that do not change CLI behavior.
- Tests that lock down existing behavior.
- Dependency/config updates that are behavior-neutral.

If the change is ambiguous, OpenSpec is the safer default.

## Minimal workflow (create → validate → implement → archive)

1) **Create a change**
- Pick a unique verb-led `change-id` (kebab-case): `add-…`, `update-…`, `remove-…`, `refactor-…`
- Create: `openspec/changes/<change-id>/`
- Add:
  - `proposal.md` (Why / What / Impact)
  - `tasks.md` (implementation checklist; keep it accurate)
  - `design.md` (only if there are non-trivial decisions)
  - `specs/<capability>/spec.md` delta(s) with:
    - `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`
    - at least one `#### Scenario:` per requirement

2) **Validate**
- Run: `openspec validate <change-id> --strict --no-interactive`
- Fix any spec/delta issues before implementation.

3) **Implement**
- Follow `tasks.md` sequentially.
- Update/extend tests (golden snapshots under `tests/golden/` when output stability matters).
- Keep `docs/SPEC.md` aligned with the implementation (this file is the single authoritative contract).

4) **PR + review**
- Link the related issue(s).
- Include evidence (`cargo test --all --locked`, relevant `--json` outputs for contract work).

5) **Archive (separate PR)**
- After the change lands, archive the proposal:
  - `openspec archive <change-id> --yes`
- Validate again (`openspec validate --all --strict --no-interactive`) and merge the archive PR.

## Common pitfalls

- Updating `docs/SPEC.md` but forgetting OpenSpec deltas (or vice versa).
- Changing `--json` output shape or semantics without updating docs + golden tests.
- Introducing a new stable error code but not updating `docs/ERROR_CODES.md`.
- Adding a new mutating command without updating `src/cli/util.rs` (`MUTATING_COMMAND_IDS`) and guardrails tests.
- Shipping output changes without snapshot/contract tests (especially ordering stability).
