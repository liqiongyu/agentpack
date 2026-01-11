# Change: Add `agentpack preview` composite command

## Why
`plan` and `diff` are often run together during review and troubleshooting. A single, AI-friendly command reduces friction while keeping the workflow read-only.

## What Changes
- Add `agentpack preview` which always runs `plan`, and optionally includes `diff` via `--diff`.
- `preview` is read-only and never requires `--yes`.
- In `--json`, return `data.plan` and (when `--diff`) `data.diff`, plus `warnings[]`.

## Acceptance
- `agentpack preview` runs without writing outputs.
- `agentpack preview --json` is parseable and includes `data.plan`.
- `agentpack preview --diff --json` includes both `data.plan` and `data.diff`.
