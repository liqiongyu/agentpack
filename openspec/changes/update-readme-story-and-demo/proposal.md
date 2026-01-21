# Change: Update README with scenarios + full loop demo

## Why
The README is the first-touch surface for most users. Adding concrete scenarios, a real end-to-end loop, and an explicit “why not X” entry reduces confusion, improves onboarding, and makes Agentpack’s boundaries clearer.

## What Changes
- Add 3 real-world scenarios to `README.md` and `README.zh-CN.md`.
- Add a single “full loop” demo (update → preview --diff → deploy --apply → status → rollback).
- Link to the dotfiles-manager comparison page and the 5-minute demo.

## Impact
- Affected specs: `agentpack-cli` (docs/onboarding surface)
- Affected code/docs: `README.md`, `README.zh-CN.md`
- Compatibility: docs-only; no CLI/JSON behavior changes
