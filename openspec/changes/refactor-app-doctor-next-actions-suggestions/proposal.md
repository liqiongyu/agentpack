# Change: Refactor doctor next_actions suggestions to shared helper

## Why
CLI `doctor` and the MCP `doctor` tool both generate `next_actions` suggestions by parsing the same `DoctorReport` signals (e.g. root suggestions like `create directory: mkdir -p ...` and the `doctor --fix` suggestion when `.gitignore` is missing the target manifest ignore rule).

Today this suggestion logic is duplicated across the two surfaces, which increases the risk of drift over time.

## What Changes
- Centralize the doctor `next_actions` suggestion logic under the app layer.
- Update CLI `doctor` and the MCP `doctor` tool to reuse the shared helper.

## Impact
- Affected specs: `agentpack-cli`, `agentpack-mcp` (doctor next_actions suggestions).
- Affected code:
  - `src/app/` (new shared helper)
  - `src/cli/commands/doctor.rs` (dedupe)
  - `src/mcp/tools.rs` (dedupe)
- No user-facing behavior change expected.
