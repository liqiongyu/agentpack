# Change: add ERROR_CODES.md consistency check

## Why

`docs/ERROR_CODES.md` is a stable registry for `--json` automation (`errors[0].code`). Drift between the registry and the actual codes emitted by the CLI breaks tooling and agents.

We need a CI guardrail that fails when the docs registry is missing/has extra codes compared to the codes emitted by Agentpack in `--json` mode.

## What Changes

- Add a deterministic CI check (Rust test) that compares:
  - codes documented in `docs/ERROR_CODES.md`
  - codes emitted by the CLI in `--json` mode (i.e., `UserError` codes + the `E_UNEXPECTED` fallback)
- Provide a clear failure message with remediation (update the docs registry).

## Non-Goals

- Do not change any existing error code behaviors.
- Do not change the JSON envelope schema.

## Impact

- Affected specs: `openspec/specs/agentpack-cli/spec.md`
- Affected tests: new test under `tests/`
- Affected docs: none (unless drift is found)
