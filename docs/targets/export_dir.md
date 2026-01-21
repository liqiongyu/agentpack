# Target mapping: `export_dir` (experimental)

> Maturity: **experimental** (feature-gated: build with `--features target-export-dir`)

## 1) Overview

- **Target id**: `export_dir`
- **Supported scope**: `user|project|both`
  - If `scope: both`, outputs are written under `<export_root>/user/` and `<export_root>/project/`.
  - If only one scope is enabled, outputs are written directly under `<export_root>/`.
- **Primary user value**: export compiled assets into a deterministic filesystem tree for inspection and integration with other tooling.
- **Non-goals**:
  - No direct integration with a specific editor/agent tool’s discovery rules
  - No auto-install / post-processing steps (pure file writes + manifests + rollback)

## 2) Managed roots

The target manages one root directory per enabled scope:

- `scope: user` → `<export_root>`
- `scope: project` → `<export_root>`
- `scope: both` → `<export_root>/user` and `<export_root>/project`

Each managed root writes/updates:
- `<root>/.agentpack.manifest.export_dir.json`

`scan_extras`:
- default: `true` (so `status` can report unmanaged “extra” files under the root)
- configurable via `targets.export_dir.options.scan_extras`

## 3) Module → output mapping

### `instructions`

- Output path(s):
  - `<root>/AGENTS.md`
- Composition rules:
  - If multiple `instructions:*` modules contribute, the file is aggregated using per-module section markers (preserves attribution for `evolve propose`-style mapping).

### `skill`

- Output path(s):
  - `<root>/skills/<skill_name>/...`
- `<skill_name>` derivation:
  - Prefer `skill:<name>` suffix
  - Fallback: filesystem-safe sanitization of `module_id`

### `prompt`

- Output path(s):
  - `<root>/prompts/<filename>.md`
- Filename rules:
  - Uses the first file in the prompt module directory (same convention as other targets).

### `command`

- Output path(s):
  - `<root>/commands/<filename>.md`
- Filename rules:
  - Uses the first file in the command module directory.

## 4) Target options and environment variables

Options under `targets.export_dir.options`:

- `root` (string, **required**): export root directory
  - `~` is supported (tilde expansion)
  - relative paths are resolved relative to `<project_root>`
- `scan_extras` (bool, default: `true`): whether `status` scans unmanaged files under the root

No environment variables are currently defined for this target.

## 5) Validation rules (linting / safety)

- If `targets.export_dir.options.root` is missing or empty, agentpack fails with stable error code `E_CONFIG_INVALID`.

## 6) Examples

### Minimal manifest example

```yaml
version: 1

profiles:
  default:
    include_tags: ["base"]

targets:
  export_dir:
    mode: files
    scope: project
    options:
      root: "./out/agent-assets"

modules:
  - id: instructions:base
    type: instructions
    tags: ["base"]
    targets: ["export_dir"]
    source:
      local_path:
        path: modules/instructions/base
```

### Output example (paths)

- `instructions:base` → `<export_root>/AGENTS.md`
- `skill:demo` → `<export_root>/skills/demo/...`
- `prompt:hello` → `<export_root>/prompts/hello.md`
- `command:hello` → `<export_root>/commands/hello.md`

## 7) Migration notes

None (experimental target, feature-gated, no compatibility guarantees yet).

## 8) Conformance checklist

- See `tests/conformance_targets.rs` (`conformance_export_dir_smoke`).
