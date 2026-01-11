---
description: "Check Agentpack environment and target roots (doctor)"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack doctor --json")
---

Run Agentpack doctor to check environment, target roots, and git hygiene.

!bash
agentpack doctor --json
