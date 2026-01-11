# agentpack-operator

Operate Agentpack (plan/diff/deploy/status/rollback) safely and reproducibly.

## What you can do
- Self-check environment: `agentpack doctor --json`
- Preview changes: `agentpack plan --json` and `agentpack diff --json`
- Apply changes: `agentpack deploy --apply --yes --json`
- Verify drift: `agentpack status --json`
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
