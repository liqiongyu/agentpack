# Change: add init --guided

## Why

Day-1 adoption needs a “zero-knowledge” path to create a valid `agentpack.yaml` without reading docs. A minimal interactive wizard reduces friction while keeping the existing `init` behavior intact.

## What Changes

- Add `agentpack init --guided`:
  - Only runs in a real TTY (stdin and stdout must be terminals).
  - Asks a small set of questions (targets, scope, optional bootstrap) and writes a manifest that can run through `update → preview --diff → deploy --apply --yes`.
- In non-TTY contexts, `--guided` fails with a clear message; in `--json` mode it returns a stable error code so automation can branch safely.

## Non-Goals

- Do not redesign the manifest schema.
- Do not add new targets.
- Do not change default behavior of `agentpack init` without `--guided`.

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`
- Affected code: `src/cli/args.rs`, `src/cli/commands/init.rs`
- Affected docs: `docs/CLI.md`, `docs/WORKFLOWS.md`, `docs/ERROR_CODES.md` (new error code)
- Tests: add integration tests for non-TTY failure + generated manifest validity
