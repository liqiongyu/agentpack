# agentpack-cli (delta)

## ADDED Requirements

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
