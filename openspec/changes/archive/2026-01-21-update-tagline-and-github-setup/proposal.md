# Change: Update tagline and GitHub setup checklist

## Why
New users often see the repo “About” blurb and README top section before anything else. If the tagline is missing or inconsistent across languages, the value proposition is harder to understand and the messaging drifts over time.

## What Changes
- Add a single, consistent EN/ZH tagline at the top of `README.md` and `README.zh-CN.md`.
- Extend `docs/GITHUB_SETUP.md` with a copy-paste checklist for GitHub “About” fields (tagline/topics/website), matching the README tagline.

## Impact
- Affected specs: `agentpack-cli` (docs/discovery surface)
- Affected code/docs: `README.md`, `README.zh-CN.md`, `docs/GITHUB_SETUP.md`
- Compatibility: docs-only; no CLI/JSON behavior changes
