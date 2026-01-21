## ADDED Requirements

### Requirement: Feature-gated experimental target `export_dir`

The system SHALL provide an experimental target adapter `export_dir`, compiled only when the `target-export-dir` feature is enabled.

The `export_dir` target SHALL write compiled outputs into a configured root directory (`targets.export_dir.options.root`) using deterministic subpaths per module type.

#### Scenario: export_dir writes outputs under a configured root
- **GIVEN** the binary is built with `--features target-export-dir`
- **AND** the manifest configures `targets.export_dir.options.root`
- **WHEN** the user runs `agentpack deploy --apply`
- **THEN** the exported root contains the expected files and a `.agentpack.manifest.export_dir.json`
