# agentpack (delta)

## ADDED Requirements

### Requirement: overlay_kind supports patch overlays
The system SHALL support an overlay kind indicator with values:
- `dir` (directory overlays; current behavior)
- `patch` (patch-based overlays)

The overlay kind indicator SHALL be stored as JSON metadata at:
`<overlay_dir>/.agentpack/overlay.json`

With format:
`{ "overlay_kind": "dir" | "patch" }`

If `overlay_kind` is not specified for an existing overlay, it SHALL be treated as `dir` (backward compatible).

For `patch` overlays, the overlay directory SHALL NOT contain normal override files (except metadata under `.agentpack/`); instead it SHALL contain patch artifacts under `.agentpack/patches/`.

Patch overlays SHALL be text-only: patches apply to UTF-8 files; binary/non-UTF8 patching is out of scope and MUST be rejected by the implementation.

#### Scenario: existing overlay defaults to dir
- **GIVEN** an overlay directory exists without an explicit `overlay_kind`
- **WHEN** desired state is computed
- **THEN** the overlay is treated as a directory overlay (`dir`)
