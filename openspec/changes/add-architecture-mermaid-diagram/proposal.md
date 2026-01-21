# Change: Add Mermaid architecture diagram

## Why
Agentpack’s core pipeline (manifest/lock/overlays → render → plan/diff → apply → snapshots/manifests/events) is easier to understand with a single diagram that GitHub can render inline. Today `docs/explanation/architecture.md` is text-only and there is no zh-CN counterpart.

## What Changes
- Add a concise Mermaid architecture diagram to `docs/explanation/architecture.md`.
- Add a zh-CN version at `docs/zh-CN/explanation/architecture.md` (diagram reused; explanation translated/summarized).
- Embed the diagram in the root `README.md` (at least one “start” surface shows it inline).

## Impact
- Affected specs: `agentpack-cli` (docs/discovery surface)
- Affected code/docs: `docs/explanation/architecture.md`, `docs/zh-CN/explanation/architecture.md`, `README.md`
- Compatibility: docs-only; no CLI/JSON behavior changes
