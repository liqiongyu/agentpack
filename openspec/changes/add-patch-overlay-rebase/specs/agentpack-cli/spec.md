# agentpack-cli (delta)

## MODIFIED Requirements

### Requirement: overlay rebase updates overlays against upstream changes
The system SHALL provide `agentpack overlay rebase <module_id>` to update an existing overlay against the current upstream module content.

The command SHALL:
- use `<overlay_dir>/.agentpack/baseline.json` as the merge base,
- update overlay edits to incorporate upstream changes using a 3-way merge, and
- update overlay baseline metadata after a successful rebase.

For `overlay_kind=dir` overlays, the command operates on overlay override files in the overlay directory.

For `overlay_kind=patch` overlays, the command SHALL:
- treat each `.agentpack/patches/<relpath>.patch` as an edit of `<relpath>`,
- compute the edited content by applying the patch to the baseline version of `<relpath>`,
- merge the edited content against the latest upstream version of `<relpath>` using a 3-way merge, and
- update the patch file so it represents a unified diff from the latest upstream content to the merged content.

When invoked with `--sparsify`, the command SHALL delete any patch file that becomes a no-op (the merged content equals the latest upstream content) and SHOULD prune now-empty parent directories under `.agentpack/patches/`.

If the merge produces conflicts for `overlay_kind=patch`, the command SHALL write conflict-marked full file content under:
`<overlay_dir>/.agentpack/conflicts/<relpath>`

#### Scenario: rebase updates an unmodified file copy
- **GIVEN** an overlay contains an unmodified copy of an upstream file (identical to the baseline)
- **AND** the upstream file changes
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --scope global`
- **THEN** the overlay file is updated so it no longer pins the old upstream content

#### Scenario: patch overlay rebase updates patch files
- **GIVEN** an overlay directory with `overlay_kind=patch`
- **AND** the overlay contains a patch file for `<relpath>`
- **AND** the upstream file `<relpath>` changes since the baseline
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --scope global`
- **THEN** the patch file is updated so it applies cleanly against the latest upstream content

#### Scenario: rebase conflict yields stable error code
- **GIVEN** an overlay edits a file that is also changed upstream since the baseline
- **AND** the changes overlap and cannot be merged cleanly
- **WHEN** the user runs `agentpack overlay rebase <moduleId> --json --yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_OVERLAY_REBASE_CONFLICT`
