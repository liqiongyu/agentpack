# Change: Bootstrap operator version stamps + status warnings (v0.4)

## Why
Bootstrap-installed operator assets (Codex skill + Claude commands) can become stale as `agentpack` evolves. New users should get an explicit warning when those assets are missing/outdated, and the templates themselves should carry an `agentpack_version` marker to make this detection reliable.

## What Changes
- Stamp operator assets with `agentpack_version: x.y.z` (frontmatter or comment) at install time.
- Expand Claude operator commands to cover the v0.4 recommended workflow (doctor/update/preview/deploy/status/explain/evolve).
- Add `agentpack status` warnings when operator assets are missing/outdated.
- Add tests for warning behavior.

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/cli.rs`, `templates/`
