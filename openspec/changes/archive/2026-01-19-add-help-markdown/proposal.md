# Change: Generate CLI reference via `agentpack help --markdown`

## Why

`docs/reference/cli.md` is a high-churn document that must stay consistent with the Clap CLI definition. Manual edits inevitably drift and require ongoing doc-sync tests.

Generating the reference from the CLI definition reduces maintenance cost and makes drift obvious.

## What Changes

- Add `agentpack help --markdown` to output a deterministic Markdown reference for the CLI.
- Make `docs/reference/cli.md` generated (or semi-generated) from that output.
- Add a CI check that fails when the committed reference is out of sync with the generator output.

## Impact

- Affected specs: `agentpack-cli`
- Affected code: CLI help command, docs/tests
- Affected docs:
  - `docs/reference/cli.md`
