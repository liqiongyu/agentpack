---
description: "Explain drift sources for this repo (explain status)"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack explain status --json")
---

Explain why drift exists for the current project (module/overlay sources).

!bash
agentpack explain status --json
