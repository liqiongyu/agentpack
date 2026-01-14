---
description: "Show Agentpack plan for this repo"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack plan --json")
---

Compute the plan (what would change) without writing files.

!bash
agentpack plan --json

If you need diff details, run `/ap-diff` or `/ap-preview`.
