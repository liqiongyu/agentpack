# Change: doctor --fix confirmation semantics in --json mode

## Why

`agentpack doctor --fix` is a write-capable operation (it can update `.gitignore`). In `--json` mode, write-capable operations must require explicit `--yes` and return a stable `E_CONFIRM_REQUIRED` error otherwise.

We want to ensure `doctor --fix` follows the same guardrail pattern as other mutating commands and is covered by guardrails tests.

## What Changes

- Ensure `doctor --fix --json` requires `--yes` consistently via the central mutating guardrail helper.
- Add guardrails test coverage for `doctor --fix` in `--json` mode.

## Non-Goals

- No changes to the underlying doctor checks or what gets fixed.

## Impact

- Affected specs: `openspec/specs/agentpack/spec.md`
- Affected code: `src/cli/commands/doctor.rs`
- Affected tests: `tests/cli_guardrails.rs`
