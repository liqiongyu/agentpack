<!-- agentpack_version: {{AGENTPACK_VERSION}} -->

# agentpack-operator

Operate Agentpack (plan/diff/deploy/status/rollback) safely and reproducibly.

## What you can do
- Self-check environment: `agentpack doctor --json`
- Update sources: `agentpack update --yes --json`
- Preview changes: `agentpack preview --json` (or `agentpack plan --json` + `agentpack diff --json`)
- Apply changes: `agentpack deploy --apply --yes --json`
- Verify drift: `agentpack status --json`
- Explain why things look the way they do: `agentpack explain plan|diff|status --json`
- Record outcomes and score modules: `agentpack record` + `agentpack score`
- Propose overlay updates from drift: `agentpack evolve propose --yes --json`
- Roll back: `agentpack rollback --to <snapshot_id>`

## Safety rules
1. Never run `deploy --apply` unless the user explicitly asked to apply.
2. Always show the `plan` + (optional) `diff` summary first.
3. Prefer `--json` for machine parsing; keep outputs small and scoped.
4. Use `--repo`, `--profile`, `--target` when operating outside defaults.

## Typical workflow

### 0) Doctor
```bash
agentpack doctor --json
```

### 0.5) Update (optional)
```bash
agentpack update --yes --json
```

### 1) Plan
```bash
agentpack plan --json
```

### 2) Diff (optional)
```bash
agentpack diff --json
```

### 3) Apply (only with user approval)
```bash
agentpack deploy --apply --yes --json
```

### 4) Status
```bash
agentpack status --json
```

## AI-first feedback loop (optional)

### Explain (optional)
```bash
agentpack explain status --json
```

### Record + score (optional)
Record a lightweight execution event (example):
```bash
echo '{"module_id":"command:ap-deploy","success":true}' | agentpack record
agentpack score
```

### Evolve overlays (optional)
Capture drifted deployed files into overlays (creates a local git branch):
```bash
agentpack evolve propose --yes --json
```
