# Change: Add custom CodeQL workflow

## Why
GitHub’s CodeQL “default setup” currently fails for this repository (dynamic runs error during CodeQL init), so we lose a useful layer of static analysis and also risk recurring CI noise.

We want CodeQL alerts, but we need a repo-owned workflow that is stable for this repository.

## What Changes
- Add a custom GitHub Actions workflow that runs CodeQL analysis for:
  - Rust (the agentpack CLI)
  - GitHub Actions workflows
- Document CodeQL as a first-class security check in this repo.

## Impact
- Affected specs: `agentpack` (repo-quality guardrail).
- Affected code:
  - `.github/workflows/codeql.yml` (new)
  - `docs/SECURITY_CHECKS.md` (update)
  - `docs/GITHUB_SETUP.md` (update)
- No CLI behavior changes; no `--json` contract changes.
