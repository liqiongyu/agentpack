## ADDED Requirements

### Requirement: User docs cover patch overlays

The repository SHALL document patch overlays (`overlay_kind=patch`) in the user-facing overlays documentation and CLI reference, including how to create a patch overlay and how failures/conflicts are reported.

#### Scenario: Docs describe patch overlay creation and constraints
- **WHEN** a user reads the overlays docs and CLI reference
- **THEN** they can find `agentpack overlay edit --kind patch`
- **AND** the docs explain that patch overlays are for UTF-8 text files and store unified diffs under `.agentpack/patches/`

#### Scenario: Docs describe failure/conflict handling
- **WHEN** a patch cannot be applied during plan/deploy
- **THEN** the docs mention `E_OVERLAY_PATCH_APPLY_FAILED`
- **AND** they mention that `overlay rebase` conflicts return `E_OVERLAY_REBASE_CONFLICT` and write conflict artifacts under `.agentpack/conflicts/`
