# Demo assets (reproducible)

This folder contains reproducible CLI demo assets for Agentpack.

## Files

- `demo.tape`: VHS script (source of truth)
- `demo.gif`: generated output (do not hand-record)

## Regenerate

Prereqs:
- `vhs` (CLI recorder)
- `jq` (for extracting `snapshot_id` in the demo)

Install VHS (macOS):

```bash
brew install vhs
```

Then regenerate from the repo root:

```bash
just demo-gif
```

Notes:
- The tape runs Agentpack in a temporary `HOME` and `AGENTPACK_HOME`.
- The demo uses a copied example repo from `docs/examples/minimal_repo`.
