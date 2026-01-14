# agentpack-cli (delta)

## ADDED Requirements

### Requirement: patch overlays declare overlay_kind via metadata
For patch overlays, the overlay directory SHALL declare `overlay_kind=patch` via JSON metadata at:
`<overlay_dir>/.agentpack/overlay.json`

With format:
`{ "overlay_kind": "dir" | "patch" }`

#### Scenario: overlay_kind is read from overlay.json
- **GIVEN** an overlay directory exists
- **AND** `<overlay_dir>/.agentpack/overlay.json` contains `{ "overlay_kind": "patch" }`
- **WHEN** the user runs `agentpack plan`
- **THEN** patch overlay application is enabled for that overlay directory

### Requirement: patch overlay directory layout
For `overlay_kind=patch`, the overlay directory SHALL store patches under:
`<overlay_dir>/.agentpack/patches/<relpath>.patch`

Where:
- `<relpath>` is the POSIX-style relative path within the upstream module root (no absolute paths; no `..`).
- Each patch file represents a unified diff against the corresponding upstream file.

If both patch artifacts (`.agentpack/patches/...`) and directory override files are present in the same overlay directory, the system SHOULD treat it as a configuration error (kind conflict).

#### Scenario: patch file path is derived from upstream relpath
- **GIVEN** a module file at relative path `skills/foo/SKILL.md`
- **WHEN** a patch overlay is used
- **THEN** the patch is stored at `.agentpack/patches/skills/foo/SKILL.md.patch`

### Requirement: patch overlay apply failures return stable error code
When a patch overlay cannot be applied cleanly during desired-state generation, the CLI MUST fail with stable error code `E_OVERLAY_PATCH_APPLY_FAILED`.

#### Scenario: patch does not apply
- **GIVEN** an overlay directory with `overlay_kind=patch`
- **AND** the overlay contains a patch file that does not apply cleanly to the upstream file
- **WHEN** the user runs `agentpack plan --json`
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_OVERLAY_PATCH_APPLY_FAILED`
