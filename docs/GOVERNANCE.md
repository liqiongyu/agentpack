# Governance layer (opt-in only)

This document defines the **isolation boundary** for Agentpack’s future organization / team governance features.

The guiding principle is strict:

> Governance MUST be explicit opt-in and MUST NOT change personal-user defaults.

If you never use governance, you should be able to pretend it does not exist.

## Goals

Governance is for teams that want to enforce “AI coding asset hygiene” and distribution rules, for example:
- required/forbidden operator assets (skills / slash commands),
- minimum `allowed-tools` constraints for Claude Code commands,
- “dangerous defaults” prevention (no implicit apply),
- organization-specific distribution policies (what can be deployed where).

## Non-goals

- No changes to core deploy semantics (plan/diff/deploy/rollback) for personal users.
- No hidden “auto policy enforcement” in core commands.
- No mandatory new config files for existing users.

## Isolation decision

We choose **Option A: isolated subcommand + isolated config file**.

### 1) Separate subcommand namespace

Governance features live under a dedicated subcommand:

```bash
agentpack policy ...
```

Core commands remain unchanged:
- `agentpack plan|diff|deploy|status|doctor|...` MUST NOT read governance config.

### 2) Separate governance config file

Governance uses a separate config file (name is stable, location is explicit):
- `repo/agentpack.org.yaml`

This file is **only** read by `agentpack policy ...`.

Rationale:
- Keeps personal config (`agentpack.yaml`) clean and focused.
- Makes opt-in obvious in code review and CI.
- Allows governance to evolve without destabilizing core config parsing.

### 3) CI-friendly by design

Governance commands are designed to run in CI:
- `agentpack policy lint` is read-only and returns machine-readable `--json` output.
- A failing lint SHOULD be used as a CI gate for org-managed repos.

## Policy lint (v1 scope)

Run policy lint against a repository (default: `$AGENTPACK_HOME/repo`):

```bash
agentpack policy lint --json
```

In CI, it is common to lint the current checkout:

```bash
agentpack --repo . policy lint --json
```

If violations are found, the command exits non-zero and returns `E_POLICY_VIOLATIONS` in `errors[0].code` (details include an issues list).

## Future roadmap (high-level)

The governance layer may evolve in stages:

1) **Policy lint** (read-only):
   - Validate operator assets (frontmatter completeness, allowed-tools, etc.).
   - Detect dangerous patterns.

2) **Policy packs** (distribution + version pinning):
   - `agentpack.org.yaml` references a local or git-sourced policy pack.
   - A lock strategy ensures auditability and reproducibility.

3) **Org distribution policy spec**:
   - Define minimum fields + stable error code strategy for policy violations.
   - Must remain scoped to `agentpack policy ...`; core deploy remains unchanged.
