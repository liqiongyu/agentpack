# Target mapping template

Goal: make target additions predictable, reviewable, and testable.

Use this template when proposing or adding a new target adapter (e.g., `zed`, `jetbrains`, …).

## 1) Overview

- **Target id**: `<target_id>` (kebab-case)
- **Supported scope**: `user|project|both` (and any constraints)
- **Primary user value**: what problem this target solves, concretely
- **Non-goals**: what this target explicitly does not do

## 2) Managed roots

List every managed root (each root MUST write/update `.agentpack.manifest.<target>.json`):

- Root A: `<path>` (how it’s computed; what it contains)
- Root B: `<path>` …

Notes:
- If the target uses project identity, define how `project_root` is determined.
- Specify whether each root uses `scan_extras=true|false`.

## 3) Module → output mapping

For each supported module type, define the mapping rules:

### `instructions`

- Output path(s):
  - `<root>/<path>`
- Composition rules:
  - single vs multi-module aggregation
  - if aggregated: how attribution is preserved (e.g., per-module section markers)

### `skill`

- Output path(s):
  - `<root>/<path>`
- How the skill name is derived (module id parsing + filesystem-safe fallback)
- Copy semantics (copy directory; no symlinks)

### `prompt`

- Output path(s):
  - `<root>/<path>`
- Filename normalization rules (extensions; discovery conventions)

### `command`

- Output path(s):
  - `<root>/<path>`
- Naming rules (how a filename maps to the command name)

## 4) Target options and environment variables

Document all options under `targets.<target_id>.options`:

- `option_name`: type, default, constraints, effect

If the target supports env overrides, document them here:

- `ENV_VAR_NAME`: meaning, precedence vs config, examples

## 5) Validation rules (linting / safety)

List any validation rules enforced by agentpack for this target:

- Required frontmatter fields (if applicable)
- Forbidden patterns or dangerous defaults
- Unsupported combinations (e.g., scope restrictions)

Also include error behavior:
- Stable `--json` error codes to use for common failures

## 6) Examples

### Minimal manifest example

```yaml
version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  <target_id>:
    mode: files
    scope: <user|project|both>
    options:
      # ...

modules:
  # ...
```

### Output example (paths)

- `instructions:<name>` → `<path>`
- `skill:<name>` → `<path>`
- …

## 7) Migration notes

If users already have existing files/config:

- What file locations are expected to already exist?
- Any rename rules or backwards-compat patterns?
- How to roll back safely (snapshots + manifests)

## 8) Conformance checklist

Add conformance tests (see `TARGET_CONFORMANCE.md`) and link them here:

- [ ] Deploy/apply writes manifests per root
- [ ] Safe delete: only manifest-managed files are deleted
- [ ] Status drift: missing/modified/extra works (extras are never auto-deleted)
- [ ] Rollback restores create/update/delete effects
- [ ] JSON envelope + stable error codes remain stable
