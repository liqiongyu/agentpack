---
description: "Propose overlay updates from drift (evolve propose)"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack evolve propose --yes --json")
---

Capture drifted deployed files into overlays and create a proposal branch.

!bash
agentpack evolve propose --yes --json
