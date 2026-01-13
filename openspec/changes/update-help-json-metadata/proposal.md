# Change: Enhance `agentpack help --json` metadata

## Why
`agentpack help --json` is a stable, machine-consumable entrypoint for agents/scripts. Today it uses a partially hard-coded command list and does not expose command argument metadata or whether a command truly supports `--json` (e.g. `completions` currently errors in `--json` mode). This makes automation more fragile and increases maintenance drift risk.

## What Changes
- Enrich `agentpack help --json` output:
  - Add `supports_json` to each `data.commands[]` item.
  - Add `args[]` metadata to each `data.commands[]` item (command-specific args; global args excluded).
  - Add a top-level `data.global_args[]` list for global flags/options.
- Update `agentpack schema --json` to document the new self-description fields.
- Update documentation to reflect the enriched self-description contract.

## Impact
- Affected specs: `openspec/specs/agentpack-cli/spec.md` (help/schema contract)
- Affected docs: `docs/JSON_API.md` (self-description section)
- Affected code: `src/cli/commands/help.rs`, `src/cli/commands/schema.rs`
- Compatibility: additive only (no `schema_version` bump).
