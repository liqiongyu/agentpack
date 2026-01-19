# Change: Add docs/index.md as the single user entrypoint

## Why
New users currently have multiple competing “start here” documents. A single, task-oriented entrypoint improves discoverability and reduces onboarding friction.

## What Changes
- Add `docs/index.md` as the canonical user entrypoint (English), with a clear decision tree linking to:
  - Quickstart (from scratch)
  - Import (adopt existing assets)
  - Daily workflow
  - Automation (`--json` / MCP)
  - Reference docs (CLI/config/targets/overlays/error codes)
- Add a Simplified Chinese counterpart at `docs/zh-CN/index.md`.

## Impact
- Affected specs: `agentpack-cli`
- Affected docs: `docs/index.md`, `docs/zh-CN/index.md`
- Compatibility: no CLI/JSON behavior changes
