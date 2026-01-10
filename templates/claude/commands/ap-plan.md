---
description: "Preview Agentpack changes (plan + diff) for this repo"
allowed-tools:
  - Bash("agentpack plan --json")
  - Bash("agentpack diff --json")
---

Run a safe preview of what Agentpack would change for the current project.

!bash
agentpack plan --json

If you need per-file hashes / updates:

!bash
agentpack diff --json
