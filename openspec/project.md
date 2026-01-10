# Project Context

## Purpose
Agentpack is an AI-first local “asset control plane” that manages and deploys AI-coding assets from a single source of truth:
- Instructions (`AGENTS.md`)
- Agent Skills (`SKILL.md`)
- Codex custom prompts (`~/.codex/prompts`)
- Claude Code slash commands (`.claude/commands`)

The v0.1 goal is a reproducible, auditable workflow: `plan -> diff -> apply -> validate -> snapshot -> rollback`, with copy/render deployment (no symlinks by default).

## Tech Stack
- Language: Rust (`edition = 2024`, pinned toolchain via `rust-toolchain.toml`, MSRV via `Cargo.toml` `rust-version`).
- CI: GitHub Actions (`.github/workflows/ci.yml`) running `cargo fmt`, `cargo clippy`, `cargo test --locked`, plus RustSec audit.
- Dev tooling: `pre-commit` hooks (`.pre-commit-config.yaml`) for fmt/clippy and basic hygiene.
- Planned core crates (when implementation starts): `clap`, `serde/serde_json/serde_yaml`, `thiserror`/`anyhow`, `tracing`, `similar` (diff), `sha2` (hashing). Git integration may use `git2` or shelling out to `git` (must stay cross-platform).

## Project Conventions

### Code Style
- Formatting: `cargo fmt --all` (rustfmt defaults); never hand-format.
- Linting: `cargo clippy --all-targets --all-features -- -D warnings`; keep warnings at zero.
- Safety: forbid unsafe (`#![forbid(unsafe_code)]`); avoid `unwrap()`/`expect()` in production paths.
- Naming: modules `snake_case`, types/traits `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- CLI: flags in `kebab-case`; `--json` output is a stable contract (see `docs/SPEC.md`).

### Architecture Patterns
Follow `docs/ARCHITECTURE.md`:
- Core engine is deterministic and side-effect-minimal; adapters own target-specific filesystem rules.
- Three layers of state: config repo (audited), store/cache (reproducible), deployed outputs (rebuildable).
- Deploy uses staging + atomic replace where possible; always create snapshots to enable rollback.

### Testing Strategy
- Unit tests close to code (`#[cfg(test)]`); integration tests in `tests/`.
- Prefer golden tests for adapter `plan` output stability (see `docs/BACKLOG.md`).
- Tests must be deterministic: no network, use fixtures and temp directories.
- CI/PRs run tests with `--locked` to ensure reproducible dependency resolution.

### Git Workflow
- Use PRs to `main`; keep changes small and reviewable.
- Commit messages: Conventional Commits (e.g., `feat(cli): add plan --json`).
- GitHub operations use GitHub CLI (`gh`) instead of the web UI (issues/PRs/releases).
- For feature/architecture work: use OpenSpec proposals under `openspec/changes/<change-id>/` and run `openspec validate <change-id> --strict` before implementation; archive completed changes per `openspec/AGENTS.md`.

## Domain Context
Agentpack’s domain is “deploying assets into tool-specific discovery locations” (Codex + Claude Code). Key concepts:
- Manifest (`agentpack.yaml`) declares modules/profiles/targets.
- Lockfile (`agentpack.lock.json`) pins resolved versions and file hashes for reproducibility.
- Overlays allow user/project customization without modifying upstream modules.

## Important Constraints
- Stability over cleverness: default deployment is copy/render (no symlink reliance).
- Safety: never execute third-party scripts during deploy; always show diffs before apply (unless explicitly skipped).
- Deletions must be constrained to files managed by agentpack (avoid removing user-owned files).
- Cross-platform correctness: path handling, hashing, and file enumeration must be deterministic.
- `--json` output is treated as an API contract (backward compatible changes only).

## External Dependencies
- Codex CLI and its discovery rules for `AGENTS.md`, skills, and prompts.
- Claude Code and its discovery rules for `.claude/commands` (and allowed-tools frontmatter).
- Git sources for modules (fetch/checkout of pinned refs); GitHub Actions, Dependabot, and RustSec advisory DB for maintenance.
