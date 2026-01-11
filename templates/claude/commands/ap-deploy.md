---
description: "Apply Agentpack changes for this repo (deploy --apply)"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack deploy --apply --yes --json")
---

Apply Agentpack changes for the current project. This writes files and creates a snapshot for rollback.

!bash
agentpack deploy --apply --yes --json
