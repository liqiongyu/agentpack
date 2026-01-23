# Demo: 5 minutes to value (safe, no real writes)

> Language: English | [Chinese (Simplified)](../zh-CN/tutorials/demo-5min.md)

Goal: run a safe demo that shows a real plan/diff without touching your real home directory.

## Run

From the repo root:

```bash
./scripts/demo_5min.sh
```

On Windows (PowerShell):

```powershell
pwsh -NoProfile -File .\\scripts\\demo_5min.ps1
```

The script prefers (in order):
- `AGENTPACK_BIN` (if you set it to a path)
- `agentpack` on your PATH
- `cargo run` from this repository (if Rust is installed)

## What it does

- Creates a temporary `HOME` and a temporary `AGENTPACK_HOME`
- Copies `docs/examples/minimal_repo` into a temp workspace
- Runs:
  - `agentpack doctor --json`
  - `agentpack preview --diff --json`

It does **not** use `--apply`, so it should not write any target files. Even if a target path is resolved, it’s inside the temporary `HOME`.

## Next steps

- Create your own config repo:
  - `agentpack init`
  - `agentpack update`
  - `agentpack preview --diff`
- Apply for real (explicit confirmation):
  - `agentpack deploy --apply --yes`
- Roll back if needed:
  - `agentpack rollback --to <snapshot_id>`
