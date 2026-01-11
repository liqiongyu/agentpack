---
description: "Show Agentpack diff summary for this repo"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack diff --json")
---

Show a diff summary (hash-based) for the current project.

!bash
agentpack diff --json
