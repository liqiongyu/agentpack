# Change: Add overlays how-to (sparse/materialize/rebase)

## Why

Overlays are a core daily workflow. We already document overlay concepts and commands, but we lack a task-oriented how-to that walks users through the common sparse/materialize/rebase loop (including when patch overlays are a better fit).

This closes the remaining doc gap called out in `docs/dev/roadmap.md` (M0-DOC-001).

## What Changes

- Add an overlays how-to doc focused on the “create sparse → optional materialize → rebase after upstream updates” workflow.
- Link it from the docs entrypoints.

## Impact

- Affected specs: `agentpack-cli` (docs coverage)
- Affected docs: `docs/howto/overlays-create-sparse-materialize-rebase.md` (and possibly the zh-CN equivalent)
- Compatibility: no CLI/JSON behavior changes
