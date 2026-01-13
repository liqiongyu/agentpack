# Change: Add `init --git` convenience flag

## Why
Many users want the config repo created by `agentpack init` to be immediately git-backed and ready for sync.

## What Changes
- Add `agentpack init --git` to run `git init` in the repo directory.
- Write/update a minimal `.gitignore` in the repo directory (idempotent).

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/cli/args.rs`, `src/cli/commands/init.rs`
