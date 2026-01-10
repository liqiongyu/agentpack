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
- `docs/`: product/design docs (`docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/SPEC.md`, `docs/BACKLOG.md`).
- `openspec/`: proposal-driven changes; keep OpenSpec blocks intact.
- `.github/`: CI and GitHub templates.

## Build, Test, and Development Commands
- `cargo build`: build locally.
- `cargo test --all --locked`: run tests (use `--locked` in CI/PRs).
- `cargo fmt --all`: format Rust code (required).
- `cargo clippy --all-targets --all-features -- -D warnings`: lint and fail on warnings (required before PRs).
- `pre-commit install`: install git hooks (install with `uv pip install pre-commit --system` or `pipx install pre-commit`).
- `pre-commit run -a`: run all hooks; optional: `pre-commit run cargo-test -a` (manual stage).

## Coding Style & Naming Conventions
- Rust formatting: use `rustfmt` defaults; do not hand-format.
- Linting: prefer Clippy-clean code; keep warnings at zero.
- Naming: modules `snake_case`, types/traits `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- Toolchain: keep `rust-toolchain.toml`/`rust-version` aligned; avoid `unsafe` (crate forbids it).
- Errors/logging: avoid `unwrap()`/`expect()` in production; prefer `Result` + structured logs (`tracing`).
- CLI: flags in `kebab-case`; stable `--json` output with explicit error codes (see `docs/SPEC.md`).

## Testing Guidelines
- Unit tests live next to code (`#[cfg(test)]`); integration tests in `tests/`.
- Add golden tests for adapter `plan` output stability (see `docs/BACKLOG.md`).
- Keep tests deterministic (avoid network; use fixtures).

## Commit & Pull Request Guidelines
- No Git history yet; use Conventional Commits (e.g., `feat(cli): add plan --json`, `fix(codex): respect CODEX_HOME`).
- PRs should: explain intent, link issues, include evidence (logs/JSON output), and note OS tested; CI must pass (fmt/clippy/test).
- Prefer GitHub CLI for GitHub operations: `gh issue create`, `gh pr create`, `gh pr view`, `gh pr checkout`.
