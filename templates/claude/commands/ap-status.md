---
description: "Check drift between expected and deployed Agentpack outputs"
allowed-tools:
  - Bash("agentpack status --json")
---

Check for drift (missing/modified/extra files) for the current project.

!bash
agentpack status --json
