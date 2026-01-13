# Change: Add optional durability mode for atomic writes

## Why
Some users prefer stronger crash consistency guarantees during deploy/apply, even at the cost of slower writes.

## What Changes
- Add an opt-in durability mode controlled by the environment variable `AGENTPACK_FSYNC`.
- When enabled, `write_atomic` uses `sync_all` to increase the likelihood that bytes and rename are durable on disk.

## Impact
- Affected specs: `agentpack`
- Affected code: `src/fs.rs`
