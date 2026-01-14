---
name: agentpack-operator
description: Operate agentpack safely in Claude Code. Use /ap-* commands for execution; prefer --json.
agentpack_version: "{{AGENTPACK_VERSION}}"
---

# agentpack-operator

Operate Agentpack safely and reproducibly in Claude Code.

## When to use Agentpack
- The user asks to check drift, preview changes, apply changes, or explain why changes happened.
- You need a safe, scriptable workflow with stable `--json` output and guardrails.

## How to operate (Claude Code)
Prefer the `/ap-*` slash commands installed by `agentpack bootstrap`:
- `/ap-doctor`: validate environment and target roots
- `/ap-update`: update lock + fetch (mutating; user-invoked only)
- `/ap-preview`: plan + optional diff summary (read-only)
- `/ap-plan`: compute desired state plan (read-only)
- `/ap-diff`: diff desired vs current (read-only)
- `/ap-deploy`: apply changes (mutating; user-invoked only)
- `/ap-status`: compute drift (read-only)
- `/ap-explain`: explain plan/diff/status (read-only)
- `/ap-evolve`: propose overlay changes from drift (mutating; user-invoked only)

If the `/ap-*` commands are missing or outdated, run:

```bash
agentpack --target claude_code bootstrap --scope both
```

## Safety rules
1. Never run `deploy --apply` unless the user explicitly asked to apply.
2. Always show `preview`/`plan` first, and include `diff` when useful.
3. Prefer `--json` for automation and keep outputs small and scoped.
4. For mutating operations in automation, always pass `--yes` and require explicit user approval.
