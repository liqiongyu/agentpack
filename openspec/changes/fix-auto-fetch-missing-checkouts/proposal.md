# Change: Auto-fetch missing git checkouts when lockfile exists

## Why
With a lockfile present, the engine prefers lockfile-resolved git checkouts for reproducibility. Today, if the local checkout cache is missing (e.g., fresh machine, cache cleared), `plan`/materialization can fail even though the lockfile is valid. This is daily-friction and a sharp edge for AI-first usage.

## What Changes
- When a module is resolved from `agentpack.lock.json` and its git checkout directory is missing, Agentpack automatically runs a safe `ensure_git_checkout()` to populate the cache.
- This only affects missing-checkout cases; existing checkouts are reused.
- Error messages remain actionable when git operations fail.

## Scope
- In scope: git modules resolved via lockfile.
- Out of scope: new CLI flags like `--no-auto-fetch` (can be added later if needed).

## Acceptance
- A missing checkout for a locked git module no longer causes plan/materialization failures.
- Behavior is covered by tests.
