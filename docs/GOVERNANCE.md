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

## Policy packs (pinning via lockfile)

Teams can optionally reference a policy pack via `repo/agentpack.org.yaml` and pin it via a lockfile for auditability and CI reproducibility.

Example `repo/agentpack.org.yaml`:

```yaml
version: 1

policy_pack:
  source: "git:https://github.com/your-org/agentpack-policy-pack.git#ref=v1.0.0&subdir=pack"
```

Pin the policy pack (writes `repo/agentpack.org.lock.json`):

```bash
agentpack --repo . policy lock
```

When `policy_pack` is configured, `policy lint` expects the lockfile to exist and match the configured source. This keeps `policy lint` CI-friendly (no network access required).

## Org distribution policy (minimal v1)

Organizations can optionally define a minimal “distribution policy” in `repo/agentpack.org.yaml` to enforce what a repo **must** configure.

Example `repo/agentpack.org.yaml`:

```yaml
version: 1

distribution_policy:
  required_targets: ["codex", "claude_code"]
  required_modules: ["instructions:base"]
```

Enforcement:
- `agentpack policy lint` validates the policy against `repo/agentpack.yaml`:
  - `required_targets[]` MUST exist under `targets:`
  - `required_modules[]` MUST exist under `modules:` and be `enabled: true`
- Violations fail with `E_POLICY_VIOLATIONS` (CI gate).

Non-goal: this policy does not change `agentpack plan/diff/deploy/...` behavior.

## Supply chain policy (remote allowlist)

Organizations can optionally enforce a git remote allowlist for modules declared in `repo/agentpack.yaml`.

Example `repo/agentpack.org.yaml`:

```yaml
version: 1

supply_chain_policy:
  allowed_git_remotes: ["github.com/your-org/"]
  require_lockfile: true
```

Enforcement:
- `agentpack policy lint` validates that every git-sourced module in `repo/agentpack.yaml` uses a remote that matches at least one allowlist entry.
- Matching is case-insensitive and normalizes common git URL forms (e.g. `https://github.com/your-org/repo.git` and `git@github.com:your-org/repo.git`).
- When `require_lockfile=true` and enabled git modules exist, `agentpack policy lint` requires `repo/agentpack.lock.json` to exist and include entries for those modules.

## Future roadmap (high-level)

The governance layer may evolve in stages:

1) **Policy lint** (read-only):
   - Validate operator assets (frontmatter completeness, allowed-tools, etc.).
   - Detect dangerous patterns.

2) **Policy packs** (distribution + version pinning):
   - `agentpack.org.yaml` references a local or git-sourced policy pack.
   - A lock strategy ensures auditability and reproducibility.

3) **Org distribution policy spec**:
   - Implemented (minimal v1) via `distribution_policy` in `repo/agentpack.org.yaml`.
   - Scoped to `agentpack policy ...`; core deploy remains unchanged.
