<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Rust CLI (`agentpack`).
- `templates/`: embedded operator assets (Codex skill + Claude commands) used by `agentpack bootstrap`.
- `tests/`: unit tests + golden snapshots (`tests/golden/`).
- `docs/`: product/design docs (`docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/SPEC.md`, `docs/BACKLOG.md`).
- `openspec/`: proposal-driven changes; keep OpenSpec blocks intact.
- `.github/`: CI and GitHub templates.

## Specs & Contracts
- `docs/SPEC.md` is the authoritative, implementation-level contract (CLI behavior, `--json` envelopes, file formats).
- `openspec/specs/` is the OpenSpec “requirements slice”; changes go through `openspec/changes/<change-id>/...` and MUST stay consistent with `docs/SPEC.md`.
- If you change any stable output (especially `--json`), update `docs/SPEC.md` and any relevant snapshots under `tests/golden/`.

## Build, Test, and Development Commands
- `cargo fmt --all -- --check`: formatting (required).
- `cargo clippy --all-targets --all-features -- -D warnings`: lint (required).
- `cargo test --all --locked`: tests (required in CI).
- `pre-commit install`: install git hooks (one-time).
- `pre-commit run -a`: run repo hooks locally.
- `agentpack completions <shell>`: generate shell completions.

## Coding Style & Naming Conventions
- Rust formatting: use `rustfmt` defaults; do not hand-format.
- Linting: prefer Clippy-clean code; keep warnings at zero.
- Naming: modules `snake_case`, types/traits `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- Toolchain: keep `rust-toolchain.toml`/`rust-version` aligned; avoid `unsafe` (crate forbids it).
- Errors: avoid `unwrap()`/`expect()` in production; prefer `anyhow::Result` + `.context(...)`.
- CLI: flags in `kebab-case`; stable `--json` output with explicit error codes (see `docs/SPEC.md`).
- Overlays: `.agentpack/` is reserved for overlay metadata; do not deploy it.

## Testing Guidelines
- Unit tests live next to code (`#[cfg(test)]`); integration tests in `tests/`.
- Add golden tests for `plan` output stability under `tests/golden/` (see `docs/BACKLOG.md`).
- Keep tests deterministic (avoid network; use fixtures).

## Commit & Pull Request Guidelines
- Commit messages follow Conventional Commits (see `git log --oneline`) (e.g., `feat(cli): add plan --json`).
- PRs should: explain intent, link issues, include evidence (logs/JSON output), and note OS tested; CI must pass (fmt/clippy/test).
- Spec-driven work: keep `openspec/changes/<change-id>/tasks.md` checkboxes accurate; run `openspec validate <change-id> --strict`.
- Prefer GitHub CLI for GitHub operations: `gh issue create`, `gh pr create`, `gh pr view`, `gh pr checkout`.
