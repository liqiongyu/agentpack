# Change: Add integrated post-acceptance plan doc to the repo

## Why
We have an up-to-date, Codex-ready post-acceptance improvement plan document. Checking it into the repo (with required planning-doc metadata) keeps planning work auditable and makes it easier to track progress through issues/PRs.

## What Changes
- Add `docs/Agentpack_Integrated_PostAcceptance_Plan_Spec_Epics_Backlog_CodexReady.md` to git.
- Add a YAML frontmatter header (status/owner/last_updated/superseded_by/scope) as required by docs governance.

## Impact
- Affected specs: `agentpack-cli`
- Affected docs: `docs/Agentpack_Integrated_PostAcceptance_Plan_Spec_Epics_Backlog_CodexReady.md`
- Compatibility: no CLI/JSON behavior changes
