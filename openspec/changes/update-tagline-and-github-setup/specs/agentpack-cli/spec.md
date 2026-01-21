## ADDED Requirements

### Requirement: Tagline and GitHub About fields stay consistent

The repository SHALL document a single, copy-pasteable English and Chinese tagline in `README.md`, `README.zh-CN.md`, and `docs/GITHUB_SETUP.md` to keep GitHub “About” fields consistent over time.

#### Scenario: Maintainer updates GitHub About without drift
- **GIVEN** a maintainer wants to update the repository “About” tagline
- **WHEN** they follow the checklist in `docs/GITHUB_SETUP.md`
- **THEN** they can copy/paste the exact tagline that appears at the top of `README.md` and `README.zh-CN.md`
