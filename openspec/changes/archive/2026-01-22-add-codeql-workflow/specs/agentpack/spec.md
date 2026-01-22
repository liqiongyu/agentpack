## ADDED Requirements

### Requirement: Repository includes CodeQL code scanning
The repository SHALL include a GitHub Actions workflow that runs CodeQL analysis for `rust` and `actions` and uploads results to GitHub code scanning on pull requests and pushes to `main`.

#### Scenario: CodeQL runs on pull requests
- **GIVEN** a pull request with code changes
- **WHEN** CI runs
- **THEN** the CodeQL workflow executes analysis for `rust` and `actions`
- **AND** uploads results to GitHub code scanning
