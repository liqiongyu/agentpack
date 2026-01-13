# Change: Add `init --bootstrap` convenience flag

## Why
New users benefit from having operator assets installed immediately after `agentpack init`, without needing to discover and run a second command.

## What Changes
- Add `agentpack init --bootstrap` to install operator assets after initializing the repo skeleton.
- Ensure project-scope operator assets are installed into the config repo (not the caller's working directory).

## Impact
- Affected specs: `agentpack-cli`
- Affected code: `src/cli/args.rs`, `src/cli/commands/init.rs`
