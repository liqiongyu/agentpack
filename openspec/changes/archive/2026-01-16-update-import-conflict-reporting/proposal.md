# Change: update import conflict reporting

## Why

`agentpack import` is the day-1 adoption path. When an import would conflict (e.g., destination paths already exist inside the config repo, or multiple scanned candidates map to the same module id), the user/agent needs a clear, scriptable report during dry-run, before attempting `--apply`.

Today, destination path conflicts are detected only during `--apply`, which makes the workflow less predictable and harder to automate.

## What Changes

- In dry-run mode, `agentpack import --json` SHALL surface conflicts in a machine-readable way (at minimum: destination path exists in config repo; module id collisions inside the scan).
- In apply mode, conflicts SHALL continue to fail safely with a stable error code (currently `E_IMPORT_CONFLICT`), and the error details SHOULD include actionable suggestions.

## Non-Goals

- Do not add an `--adopt`/overwrite mode for import in this change.
- Do not change the `--json` envelope shape (additive fields only).

## Impact

- Affected spec: `openspec/specs/agentpack-cli/spec.md` (new requirement)
- Affected code: `src/cli/commands/import.rs`
- Affected docs: `docs/CLI.md` / `docs/JSON_API.md` (additive fields), possibly `docs/SPEC.md` notes
- Tests: extend `tests/cli_import.rs` to assert dry-run conflict reporting deterministically
