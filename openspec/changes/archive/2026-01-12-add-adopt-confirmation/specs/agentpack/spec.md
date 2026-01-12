## ADDED Requirements

### Requirement: Adopt updates require explicit confirmation
Agentpack MUST NOT overwrite an existing target file unless it is known to be managed **or** the user explicitly opts in to adopting that file.

“Known to be managed” means the file path is present in at least one of:
- the target manifest `.agentpack.manifest.json`, or
- the latest deployment snapshot (when manifests are missing).

#### Scenario: Unmanaged overwrite is refused by default
- **GIVEN** a target file already exists at an output path
- **AND** that file is not known to be managed
- **AND** the desired content differs from the existing content
- **WHEN** the user runs `agentpack deploy --apply`
- **THEN** the command fails without writing
- **AND** it reports how to re-run with explicit adopt confirmation

#### Scenario: JSON mode returns stable error code for missing adopt confirmation
- **GIVEN** an adopt update would be required
- **WHEN** the user runs `agentpack deploy --apply --json --yes` without the explicit adopt flag
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_ADOPT_CONFIRM_REQUIRED`
