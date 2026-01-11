---
description: "Check drift between expected and deployed Agentpack outputs"
agentpack_version: "{{AGENTPACK_VERSION}}"
allowed-tools:
  - Bash("agentpack status --json")
---

Check for drift (missing/modified/extra files) for the current project.

!bash
agentpack status --json

If drift is found, consider:
- `agentpack explain status --json` (why a file comes from which module/overlay)
- `agentpack evolve propose --yes --json` (capture drift into overlays on a proposal branch)
