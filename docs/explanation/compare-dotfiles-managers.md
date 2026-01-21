# Compare: Agentpack vs dotfiles managers

> Language: English | [Chinese (Simplified)](../zh-CN/explanation/compare-dotfiles-managers.md)

Agentpack is **not** a general-purpose dotfiles manager. It’s a local “asset control plane” focused on deploying AI-coding assets (AGENTS.md, skills, prompts, commands) into **tool-specific discovery locations** with safety guardrails and automation contracts.

That said, dotfiles managers and Agentpack can complement each other well.

## Quick comparison

| Dimension | GNU Stow | chezmoi | yadm | Agentpack |
| --- | --- | --- | --- | --- |
| Primary scope | Symlink farm for `$HOME` | Multi-machine dotfiles mgmt | Dotfiles-in-`$HOME` as git repo | Deploy coding-agent assets across tools |
| Deployment model | Symlinks | Copy/symlink/template (tooling) | Git + optional templating/hooks | Render desired state per **target** + apply |
| Safety + rollback | Git rollback (you manage) | Git rollback + tool features | Git rollback + tool features | Snapshots + rollback + manifest-based safe deletes |
| Targets | N/A (your layout) | N/A (mostly `$HOME`) | N/A (mostly `$HOME`) | Built-in target adapters (Codex/Claude Code/…) |
| Overlays / rebase | N/A | Templates / machine-specific config | Templates / conditionals | Overlay scopes + `overlay rebase` (3-way merge) |
| Automation contract | Shell scripting | Shell scripting | Shell scripting | Stable `--json` envelope + MCP tools (approval-gated) |

## When a dotfiles manager is the better fit

- You want to manage **all** `$HOME` dotfiles (shell/editor/git/ssh) as a single system.
- You rely heavily on **templating** or machine-specific conditionals across many configs.
- Your deployment model is primarily about **symlinks** / home-directory hygiene (and you’re happy with that risk profile).

## When Agentpack is the better fit

- You want a **single source of truth** for agent assets across multiple tools (Codex, Claude Code, Cursor, VSCode, …).
- You want **targets** (explicit mappings/validation) instead of “everything is a file in `$HOME`”.
- You need safer automation: **preview/diff-first**, explicit adopt/confirm rules, stable machine-readable outputs (`--json` / MCP).

## Recommended combination

- Use chezmoi / yadm / Stow to manage your dotfiles.
- Use Agentpack to manage only agent assets, where target mappings, overlays, and rollback semantics are useful.

## References (official docs)

- GNU Stow manual: https://www.gnu.org/s/stow/manual/stow.html
- chezmoi: https://chezmoi.io/
- yadm: https://yadm.io/
