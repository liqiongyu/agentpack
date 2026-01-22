# Change: Refactor preview diff generation into app layer

## Why
The preview per-file diff payload (`preview --json --diff` / MCP `preview {diff:true}`) is generated in multiple places today (CLI and MCP), which increases the risk of output drift and makes maintenance harder.

This change centralizes the shared “preview diff files” generation logic to keep the contract consistent across interfaces.

## What Changes
- Add a small internal “app” module that owns shared logic for preview diff file generation.
- Deduplicate common root-selection logic (`best_root_idx`) used by preview/status/manifest codepaths.

## Impact
- Affected specs: `agentpack-cli` (preview diff contract), `agentpack-mcp` (preview tool reuses the same envelope).
- Affected code:
  - `src/app/*` (new)
  - `src/cli/commands/preview.rs` (use shared helper)
  - `src/mcp/tools.rs` (use shared helper)
  - `src/handlers/status.rs`, `src/target_manifest.rs`, `src/cli/util.rs` (root selection dedupe)
- No user-facing behavior change expected.
