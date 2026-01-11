---
description: "Update Agentpack sources (lock + fetch)"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack update --yes --json")
---

Update the lockfile and fetch sources (may write to disk).

!bash
agentpack update --yes --json
