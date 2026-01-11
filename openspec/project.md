# Project Context

## Purpose
Agentpack is an AI-first local “asset control plane” that manages and deploys AI-coding assets from a single source of truth:
- Instructions (`AGENTS.md`)
- Agent Skills (`SKILL.md`)
- Codex custom prompts (`~/.codex/prompts`)
- Claude Code slash commands (`.claude/commands`)

As of v0.3, the focus is a reproducible, auditable workflow: `plan/preview -> diff -> apply -> validate -> snapshot -> rollback`, plus:
- manifest-based safety (`.agentpack.manifest.json` in target roots)
- multi-machine sync (`remote set` + `sync --rebase`)
- machine overlays (`overlays/machines/<machineId>/...` + `--machine`)
- observability + proposals (`record`/`score`/`explain`/`evolve propose`)
- lower-friction composites (`update`, `preview`) and scripting helpers (`overlay path`)
- `--json` write guardrails (`--yes` required, stable error code `E_CONFIRM_REQUIRED`)

## Canonical Specs
- `docs/SPEC.md` is the implementation-level contract (CLI behavior, `--json` envelope, file formats) and should match code + tests.
- `openspec/specs/` is the OpenSpec “requirements slice” used for proposal-driven changes; it MUST stay consistent with `docs/SPEC.md`.
- If `docs/SPEC.md` and `openspec/specs/` drift, reconcile them promptly (avoid leaving `TBD`/placeholder Purpose sections in archived specs).

## Tech Stack
- Language: Rust (`edition = 2024`, pinned toolchain via `rust-toolchain.toml`, MSRV via `Cargo.toml` `rust-version`).
- CI: GitHub Actions (`.github/workflows/ci.yml`) running `cargo fmt`, `cargo clippy`, `cargo test --locked`, plus RustSec audit.
- Dev tooling: `pre-commit` hooks (`.pre-commit-config.yaml`) for fmt/clippy and basic hygiene.
- Core crates: `clap`/`clap_complete`, `serde` + `serde_json`/`serde_yaml`, `anyhow`, `sha2` + `hex`, `walkdir`, `tempfile`, `time`, `dirs`, `similar`.
- Git integration: shelling out to `git` (cross-platform and avoids libgit2 edge cases).
- Templates: embedded at compile time via `include_str!` from `templates/` (used by `agentpack bootstrap`).

## Project Conventions

### Code Style
- Formatting: `cargo fmt --all` (rustfmt defaults); never hand-format.
- Linting: `cargo clippy --all-targets --all-features -- -D warnings`; keep warnings at zero.
- Safety: forbid unsafe (`#![forbid(unsafe_code)]`); avoid `unwrap()`/`expect()` in production paths.
- Naming: modules `snake_case`, types/traits `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- CLI: flags in `kebab-case`; `--json` output is a stable contract (see `docs/SPEC.md`):
  - write commands require `--yes`
  - missing confirmation returns `E_CONFIRM_REQUIRED` with valid JSON `ok=false`

### Architecture Patterns
Follow `docs/ARCHITECTURE.md`:
- Core engine is deterministic and side-effect-minimal; adapters own target-specific filesystem rules.
- Three layers of state: config repo (audited), store/cache (reproducible), deployed outputs (rebuildable).
- Deploy uses staging + atomic replace where possible; always create snapshots to enable rollback.
- Target manifests (`.agentpack.manifest.json`) are written into deployed roots; never treat user-owned files as managed.
- Overlay metadata lives under `.agentpack/` (e.g., overlay baselines) and must not leak into deployed outputs.

### Testing Strategy
- Unit tests close to code (`#[cfg(test)]`); integration tests in `tests/`.
- Prefer golden tests for adapter `plan` output stability (see `docs/BACKLOG.md`).
- Golden snapshot files live under `tests/golden/` and are updated intentionally (review diffs carefully).
- Tests must be deterministic: no network, use fixtures and temp directories.
- CI/PRs run tests with `--locked` to ensure reproducible dependency resolution.

### Git Workflow
- Use PRs to `main`; keep changes small and reviewable.
- Commit messages: Conventional Commits (e.g., `feat(cli): add plan --json`).
- GitHub operations use GitHub CLI (`gh`) instead of the web UI (issues/PRs/releases).
- Merges: use `gh pr merge` once CI is green; keep branches short-lived and delete after merge.
- For feature/architecture work: use OpenSpec proposals under `openspec/changes/<change-id>/` and run:
  - `openspec validate <change-id> --strict --no-interactive` before implementation
  - `openspec archive <change-id> --yes` in a separate “archive” PR after merge

## Domain Context
Agentpack’s domain is “deploying assets into tool-specific discovery locations” (Codex + Claude Code). Key concepts:
- Manifest (`agentpack.yaml`) declares modules/profiles/targets.
- Lockfile (`agentpack.lock.json`) pins resolved versions and file hashes for reproducibility.
- Overlays allow user/project customization without modifying upstream modules.

## Important Constraints
- Stability over cleverness: default deployment is copy/render (no symlink reliance).
- Safety: never execute third-party scripts during deploy; always show diffs before apply (unless explicitly skipped).
- Deletions must be constrained to files managed by agentpack (target manifests and/or snapshots; avoid removing user-owned files).
- Cross-platform correctness: path handling, hashing, and file enumeration must be deterministic.
- `--json` output is treated as an API contract (`schema_version` + backward compatible changes only).

## External Dependencies
- Codex CLI and its discovery rules for `AGENTS.md`, skills, and prompts.
- Claude Code and its discovery rules for `.claude/commands` (and allowed-tools frontmatter).
- Git sources for modules (fetch/checkout of pinned refs); GitHub Actions, Dependabot, and RustSec advisory DB for maintenance.
