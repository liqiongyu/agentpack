# Change: Make lockfile local_path repo-relative

## Why
`agentpack.lock.json` currently records absolute paths for `local_path` modules. This causes noisy diffs across machines and makes the lockfile harder to review, sync, and audit.

## What Changes
- When generating `agentpack.lock.json`, local-path modules record `resolved_source.local_path.path` as a repo-relative path (stable across machines), instead of an absolute path.
- Update `docs/SPEC.md` to document the lockfile convention.

## Impact
- Affected specs: `agentpack` (lockfile contract)
- Affected code: `src/lockfile.rs`
- Affected tests: add coverage to ensure local paths are stored repo-relative
