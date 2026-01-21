# Change: Add dotfiles-manager comparison page

## Why
Many users discover Agentpack while looking for a “dotfiles manager”. A clear, fair comparison reduces confusion, sets boundaries, and helps users decide when to use Stow/chezmoi/yadm vs when Agentpack is the right tool (or complementary).

## What Changes
- Add an explanation page comparing Agentpack with GNU Stow, chezmoi, and yadm (EN + zh-CN).
- Add a simple entrypoint link from `docs/index.md` and `docs/zh-CN/index.md`.

## Impact
- Affected specs: `agentpack-cli` (docs/discovery surface)
- Affected code/docs: new docs pages + docs index updates
- Compatibility: docs-only; no CLI/JSON behavior changes
