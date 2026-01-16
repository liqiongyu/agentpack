## Why

`agentpack evolve propose` creates a local git branch in the config repo. The current default branch name (`evolve/propose-<timestamp_nanos>`) is hard to interpret and does not encode the proposal scope or the affected module(s), which makes it harder to review and manage multiple proposals.

The roadmap calls for a more controllable and informative default branch name that includes `scope`, `module_id` (when known), and a timestamp, while still allowing explicit overrides via `--branch`.

## What changes

- Change the default `evolve propose` branch naming scheme to include:
  - scope (`global|machine|project`)
  - module attribution (`<module_id>` when filtered or a single-module proposal, otherwise `multi`)
  - timestamp (human-readable)
- Keep `--branch` behavior unchanged (explicit name wins).
- Update docs/CLI help text to reflect the new default.

## Impact

- User-visible behavior: default branch name changes (only when `--branch` is not provided).
- No JSON schema changes; output fields remain unchanged.
