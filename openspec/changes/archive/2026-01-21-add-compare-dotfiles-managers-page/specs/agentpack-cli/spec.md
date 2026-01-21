## ADDED Requirements

### Requirement: Docs clarify boundaries vs dotfiles managers

The repository SHALL include an explanation page that fairly compares Agentpack with common dotfiles managers (GNU Stow, chezmoi, yadm), highlighting strengths, trade-offs, and when not to use Agentpack.

#### Scenario: New user can choose the right tool
- **GIVEN** a new user is considering Agentpack as a dotfiles manager replacement
- **WHEN** they read the comparison page
- **THEN** they can decide whether to use Stow/chezmoi/yadm, Agentpack, or a combination, based on clear scope boundaries and referenced upstream docs
