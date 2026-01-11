---
description: "Preview Agentpack changes (plan + optional diff)"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack preview --json")
  - Bash("agentpack preview --diff --json")
---

Preview what Agentpack would change without writing files.

!bash
agentpack preview --json

If you need per-file hashes:

!bash
agentpack preview --diff --json
