# Change: Update bootstrap operator templates (AI-first guidance)

## Why
Bootstrap installs “operator assets” (Codex skill + Claude commands). Improving the guidance in these templates makes the record/score/explain/evolve loop easier to discover without changing core deploy behavior.

## What Changes
- Update `templates/codex/skills/agentpack-operator/SKILL.md` to include a recommended workflow using:
  - `plan` / `preview`
  - `deploy --apply`
  - `record` / `score`
  - `explain`
  - `evolve propose`
- Update existing Claude command templates to mention `evolve propose` (no new commands added).

## Acceptance
- Template changes are reflected in `agentpack bootstrap` outputs (no code behavior changes).
