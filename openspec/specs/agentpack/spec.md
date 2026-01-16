# agentpack Specification

## Purpose
Define the core safety and state-management guarantees of Agentpack’s deployment engine. This spec covers properties that protect user-owned files (target manifests + safe deletes), enable recoverability (snapshots/rollback), and support multi-machine and multi-scope customization (sync and overlays). It also defines critical operator guardrails for automation, especially in `--json` mode.
## Requirements
### Requirement: Target Manifest
`agentpack deploy --apply` MUST write a `.agentpack.manifest.json` file into each managed target root directory that records the managed files and their hashes.

#### Scenario: Manifest exists after deploy
- **WHEN** `agentpack deploy --apply` completes successfully
- **THEN** each target root that received files contains `.agentpack.manifest.json`
- **AND** the manifest lists every managed file with a stable relative path and `sha256` hash

### Requirement: Safe Deletes
Agentpack MUST NOT delete files that are not listed in the target manifest.

#### Scenario: Unmanaged files are preserved
- **GIVEN** a target root contains user-created files not present in the manifest
- **WHEN** `agentpack deploy --apply` runs
- **THEN** those unmanaged files remain untouched

### Requirement: Manifest-Based Status
`agentpack status` MUST compute drift using the target manifest and report managed-file drift as `modified` or `missing`, and unmanaged files as `extra` (non-fatal).

#### Scenario: Status detects drift and extras
- **GIVEN** a deployed target root with a valid manifest
- **WHEN** a managed file is modified, and an unmanaged file is added
- **THEN** `agentpack status` reports `modified` for the managed file and `extra` for the unmanaged file

#### Scenario: Unsupported manifest schema_version is tolerated
- **GIVEN** a deployed target root contains `.agentpack.manifest.json` with an unsupported `schema_version`
- **WHEN** the user runs `agentpack status`
- **THEN** the command succeeds
- **AND** it emits a warning indicating the manifest was ignored
- **AND** drift is computed using the “no manifest” fallback behavior

### Requirement: Multi-Machine Sync
Agentpack MUST provide `remote set` and `sync` commands to standardize recommended git workflows for the agentpack config repo.

#### Scenario: Sync wraps pull/rebase/push
- **WHEN** `agentpack sync --rebase` runs in a git-backed config repo with a configured remote
- **THEN** it performs a pull with rebase and then pushes (or clearly reports failures without modifying history)

### Requirement: Machine Overlays
Agentpack MUST support an optional machine overlay layer between global and project overlays, selectable via a global `--machine` flag.

#### Scenario: Machine overlay affects desired state
- **GIVEN** a file is overridden in `overlays/machines/<machine_id>/...`
- **WHEN** `agentpack plan --machine <machine_id>` runs
- **THEN** the planned content reflects the machine overlay override

### Requirement: Doctor Self-Check
Agentpack MUST provide a `doctor` command that outputs a deterministic `machine_id` and validates target paths for existence and writability with actionable guidance. Additionally, when a target root is inside a git repository, `doctor` MUST warn if `.agentpack.manifest.json` is not ignored, and `doctor --fix` MUST be able to idempotently add it to `.gitignore`.

#### Scenario: Doctor warns about committing target manifests
- **GIVEN** a target root inside a git repository
- **AND** `.agentpack.manifest.json` is not ignored
- **WHEN** the user runs `agentpack doctor`
- **THEN** the output includes a warning recommending adding it to `.gitignore`

### Requirement: JSON-mode write confirmation
When invoked with `--json`, any command that performs writes (filesystem or git) MUST require an explicit `--yes` confirmation. If `--yes` is missing, the system MUST return a JSON error with a stable code `E_CONFIRM_REQUIRED` and MUST NOT perform the write.

#### Scenario: add --json without --yes is refused
- **WHEN** the user runs `agentpack add ... --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: Reproducible materialization from lockfile
When a module is resolved from `agentpack.lock.json` and its git checkout directory is missing locally, the system MUST automatically populate the missing checkout (safe network fetch) or fail with an actionable error instructing the user to run `agentpack fetch/update`.

#### Scenario: Missing checkout is auto-fetched
- **GIVEN** `agentpack.lock.json` pins a git module to commit `<commit>`
- **AND** the local checkout directory for `<module_id, commit>` does not exist
- **WHEN** the system materializes that module as part of `plan/diff/deploy`
- **THEN** the missing checkout is populated automatically

### Requirement: Overlay editing
The system MUST support creating overlay skeletons across overlay scopes:
- `global`: `repo/overlays/<module_fs_key>/...`
- `machine`: `repo/overlays/machines/<machine_id>/<module_fs_key>/...`
- `project`: `repo/projects/<project_id>/overlays/<module_fs_key>/...`

#### Scenario: Machine overlay skeleton is created
- **GIVEN** a module `<module_id>` exists and resolves to an upstream root
- **WHEN** the user runs `agentpack overlay edit <module_id> --scope machine`
- **THEN** the directory `repo/overlays/machines/<machine_id>/<module_fs_key>/` exists
- **AND** it contains the upstream content (copied)
- **AND** it contains `.agentpack/baseline.json`

### Requirement: JSON-mode write confirmation covers all write-capable commands
When invoked with `--json`, any command that performs writes (filesystem or git) MUST require an explicit `--yes` confirmation. If `--yes` is missing, the system MUST return a JSON error with a stable code `E_CONFIRM_REQUIRED` and MUST NOT perform the write.

This includes (at minimum): `init`, `lock`, `fetch`, `overlay edit`, `remote set`, `sync`, `record`, and `rollback` (in addition to existing covered commands like `add/remove/update/deploy/bootstrap/doctor --fix/evolve propose`).

#### Scenario: init --json without --yes is refused
- **WHEN** the user runs `agentpack init --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

#### Scenario: overlay edit --json without --yes is refused
- **WHEN** the user runs `agentpack overlay edit <module_id> --json` without `--yes`
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_CONFIRM_REQUIRED`

### Requirement: Target rendering is routed via a TargetAdapter registry
The system SHALL centralize target-specific rendering and validation behind a `TargetAdapter` abstraction, so adding a new target does not require scattering conditional logic across the engine and CLI.

#### Scenario: Known targets are resolved via registry
- **GIVEN** the system supports the `codex`, `claude_code`, `cursor`, and `vscode` targets
- **WHEN** the engine renders desired state for a selected target
- **THEN** the corresponding target adapter is used to compute target roots and desired output paths

### Requirement: Target conformance tests cover critical safety semantics
The repository SHALL include conformance tests that validate critical cross-target safety semantics, including:
- delete protection (only manifest-managed paths can be deleted)
- per-root manifest write/read
- drift classification (missing/modified/extra)
- rollback restoring previous outputs

#### Scenario: conformance tests exist for built-in targets
- **GIVEN** built-in targets `codex`, `claude_code`, `cursor`, and `vscode`
- **WHEN** the test suite is run
- **THEN** conformance tests execute these semantics for all built-in targets

### Requirement: module_id is mapped to a filesystem-safe key
The system SHALL derive a stable, filesystem-safe `module_fs_key` from `module_id` and SHALL use `module_fs_key` when creating filesystem paths for module-scoped storage (e.g., overlays and cache/store directories).

#### Scenario: Windows-safe overlay directory
- **GIVEN** a module id `instructions:base`
- **WHEN** the system computes the overlay directory for that module
- **THEN** the directory path does not require using `:` as a path component

### Requirement: Legacy overlay and store paths remain usable
The system SHALL preserve backwards compatibility by preferring legacy (pre-v0.5) overlay and store paths when they already exist on disk.

#### Scenario: existing legacy overlay is used
- **GIVEN** a legacy overlay directory exists for a module id
- **WHEN** the system renders the module
- **THEN** it applies the legacy overlay directory

### Requirement: apply uses atomic writes for updates
The system SHALL avoid unnecessary pre-deletes when writing updated outputs. On platforms where atomic replacement is supported by the underlying filesystem APIs, an update SHALL replace the destination atomically.

#### Scenario: update does not require deleting the destination first
- **GIVEN** a deployed managed file exists
- **WHEN** a subsequent deploy updates that file
- **THEN** the system uses atomic replacement semantics where available

### Requirement: Optional durability mode for atomic writes
When `AGENTPACK_FSYNC` is enabled, the system SHALL increase durability of atomic writes by syncing file contents before persist and (where supported) syncing the parent directory after the atomic replace.

#### Scenario: durability mode is enabled for atomic writes
- **GIVEN** `AGENTPACK_FSYNC=1` is set in the environment
- **WHEN** the system writes a file via an atomic write path
- **THEN** it syncs file contents to disk before the atomic replace
- **AND** where supported, it syncs the parent directory after the atomic replace

### Requirement: DesiredState path conflicts are refused
If multiple modules attempt to produce different content for the same `(target, path)`, the system MUST fail fast instead of silently overwriting based on insertion order.

If the bytes are identical, the system SHOULD merge module provenance so the output can still be attributed to all contributing modules.

In `--json` mode, the system MUST return a stable error code `E_DESIRED_STATE_CONFLICT`.

#### Scenario: conflict is detected before apply
- **GIVEN** two modules render different bytes to the same output path
- **WHEN** the user runs `agentpack plan --json` (or `preview/deploy`)
- **THEN** the command exits non-zero
- **AND** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_DESIRED_STATE_CONFLICT`

### Requirement: JSON mode uses stable error codes for common failures
In `--json` mode, user-facing failures that are common and actionable MUST return stable error codes (not just `E_UNEXPECTED`) so automation can branch reliably.

At minimum, the following scenarios MUST return stable codes:
- Missing config manifest: `E_CONFIG_MISSING`
- Invalid config manifest: `E_CONFIG_INVALID`
- Unsupported config version: `E_CONFIG_UNSUPPORTED_VERSION`
- Missing lockfile when required: `E_LOCKFILE_MISSING`
- Invalid lockfile JSON: `E_LOCKFILE_INVALID`
- Unsupported lockfile version: `E_LOCKFILE_UNSUPPORTED_VERSION`
- Unsupported `--target`: `E_TARGET_UNSUPPORTED`

#### Scenario: missing config yields stable error code
- **GIVEN** `agentpack.yaml` is missing
- **WHEN** the user runs `agentpack plan --json`
- **THEN** `errors[0].code` equals `E_CONFIG_MISSING`

### Requirement: score tolerates malformed events.jsonl lines
`agentpack score` MUST tolerate malformed/partial lines in `events.jsonl` by skipping bad lines and emitting warnings, instead of failing the whole command.

Additionally, the system SHOULD provide a structured summary of how many lines were skipped and why (e.g. read errors, malformed JSON, unsupported schema_version) so operators and automation can diagnose log health.

#### Scenario: malformed line is skipped
- **GIVEN** `events.jsonl` contains both valid and invalid JSON lines
- **WHEN** the user runs `agentpack score --json`
- **THEN** the command exits successfully with `ok=true`
- **AND** `warnings` includes an entry referencing the skipped line

#### Scenario: score reports skipped counts
- **GIVEN** `events.jsonl` contains malformed JSON lines and unsupported schema versions
- **WHEN** the user runs `agentpack score --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.read_stats.skipped_total` is greater than 0
- **AND** `data.read_stats.skipped_malformed_json` is greater than 0

### Requirement: evolve propose reports skipped drift

When drift exists but cannot be safely mapped back to a single module (e.g., multi-module aggregated outputs) or when the deployed file is missing, `agentpack evolve propose` MUST report the drift as skipped instead of claiming there is no drift.

#### Scenario: missing drift is reported as skipped
- **GIVEN** an expected managed output is missing on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** stdout is valid JSON with `ok=true`
- **AND** `data.reason` equals `no_proposeable_drift`
- **AND** `data.skipped[]` contains an item with `reason=missing`
- **AND** that item contains `reason_code=missing`

### Requirement: Adopt updates require explicit confirmation
Agentpack MUST NOT overwrite an existing target file unless it is known to be managed **or** the user explicitly opts in to adopting that file.

“Known to be managed” means the file path is present in at least one of:
- the target manifest `.agentpack.manifest.json`, or
- the latest deployment snapshot (when manifests are missing).

#### Scenario: Unmanaged overwrite is refused by default
- **GIVEN** a target file already exists at an output path
- **AND** that file is not known to be managed
- **AND** the desired content differs from the existing content
- **WHEN** the user runs `agentpack deploy --apply`
- **THEN** the command fails without writing
- **AND** it reports how to re-run with explicit adopt confirmation

#### Scenario: JSON mode returns stable error code for missing adopt confirmation
- **GIVEN** an adopt update would be required
- **WHEN** the user runs `agentpack deploy --apply --json --yes` without the explicit adopt flag
- **THEN** stdout is valid JSON with `ok=false`
- **AND** `errors[0].code` equals `E_ADOPT_CONFIRM_REQUIRED`

### Requirement: events.jsonl is forward-compatible
`events.jsonl` MUST be treated as an evolvable audit log:
- Readers MUST ignore unknown top-level fields (additive changes).
- Readers MUST skip unsupported `schema_version` entries and emit warnings (do not fail the whole command).

#### Scenario: unknown fields are ignored
- **GIVEN** an `events.jsonl` entry includes additional top-level fields that are not recognized by this version
- **WHEN** the user runs `agentpack score --json`
- **THEN** the entry is parsed successfully and does not cause a failure

#### Scenario: unsupported schema_version is skipped
- **GIVEN** an `events.jsonl` entry with `schema_version` not supported by this version
- **WHEN** the user runs `agentpack score --json`
- **THEN** the entry is skipped
- **AND** a warning is emitted

### Requirement: Lockfile local paths are repo-relative
When generating `agentpack.lock.json`, the system SHALL record `resolved_source.local_path.path` as a stable, repo-relative path so the lockfile remains portable across machines.

#### Scenario: Lockfile local_path is stable across machines
- **GIVEN** a module uses a `local_path` source inside the agentpack repo
- **WHEN** the user runs `agentpack lock`
- **THEN** the lockfile stores `resolved_source.local_path.path` without embedding an absolute machine path

### Requirement: Combined instructions outputs include per-module markers
When multiple `instructions` modules contribute to a combined deployed `AGENTS.md`, the system SHALL include stable, per-module section markers so drift in the combined file can be mapped back to a specific module.

#### Scenario: Combined AGENTS.md contains markers
- **GIVEN** two `instructions` modules are enabled for the `codex` target
- **WHEN** the system renders desired state for Codex agent instructions
- **THEN** the combined `AGENTS.md` content contains a section marker per module id

### Requirement: evolve propose can map marked instructions drift back to a module
When a deployed combined `AGENTS.md` contains valid per-module section markers, `agentpack evolve propose` SHALL treat drifted sections as proposeable and generate overlay updates for the corresponding `instructions` module(s).

#### Scenario: drifted marked section becomes a propose candidate
- **GIVEN** a deployed combined `AGENTS.md` containing section markers for `instructions:one` and `instructions:two`
- **AND** only the `instructions:one` section content is edited on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** `data.candidates[]` contains an item with `module_id="instructions:one"`
- **AND** the drift is not reported as `multi_module_output` skipped for that output

### Requirement: Cursor target writes project rules
The system SHALL support a built-in `cursor` target (files mode) that renders `instructions` modules into Cursor project rule files under `.cursor/rules`.

#### Scenario: deploy writes cursor rule files and a manifest
- **GIVEN** an enabled `instructions` module targeting `cursor`
- **WHEN** the user runs `agentpack --target cursor deploy --apply`
- **THEN** at least one `.mdc` rule file exists under `<project_root>/.cursor/rules`
- **AND** `<project_root>/.cursor/rules/.agentpack.manifest.json` exists

#### Scenario: rule filenames are stable and unique
- **GIVEN** two different enabled `instructions` modules targeting `cursor`
- **WHEN** the user runs `agentpack --target cursor deploy --apply`
- **THEN** the generated filenames are distinct and derived from each module’s `module_fs_key`

### Requirement: VS Code target writes Copilot instruction and prompt files
The system SHALL support a built-in `vscode` target (files mode) that renders:
- `instructions` modules into `.github/copilot-instructions.md`, and
- `prompt` modules into `.github/prompts/*.prompt.md`.

#### Scenario: deploy writes vscode files and manifests
- **GIVEN** at least one enabled module targeting `vscode`
- **WHEN** the user runs `agentpack --target vscode deploy --apply`
- **THEN** `<project_root>/.github/copilot-instructions.md` exists when `instructions` modules are present
- **AND** `<project_root>/.github/prompts/` contains `.prompt.md` files when `prompt` modules are present
- **AND** per-root `.agentpack.manifest.json` files exist under `.github/` and `.github/prompts/`

#### Scenario: prompt filenames end with .prompt.md
- **GIVEN** a `prompt` module whose source file is named `hello.md`
- **WHEN** the user runs `agentpack --target vscode deploy --apply`
- **THEN** the deployed file name ends with `.prompt.md`

### Requirement: overlay_kind supports patch overlays
The system SHALL support an overlay kind indicator with values:
- `dir` (directory overlays; current behavior)
- `patch` (patch-based overlays)

The overlay kind indicator SHALL be stored as JSON metadata at:
`<overlay_dir>/.agentpack/overlay.json`

With format:
`{ "overlay_kind": "dir" | "patch" }`

If `overlay_kind` is not specified for an existing overlay, it SHALL be treated as `dir` (backward compatible).

For `patch` overlays, the overlay directory SHALL NOT contain normal override files (except metadata under `.agentpack/`); instead it SHALL contain patch artifacts under `.agentpack/patches/`.

Patch overlays SHALL be text-only: patches apply to UTF-8 files; binary/non-UTF8 patching is out of scope and MUST be rejected by the implementation.

#### Scenario: existing overlay defaults to dir
- **GIVEN** an overlay directory exists without an explicit `overlay_kind`
- **WHEN** desired state is computed
- **THEN** the overlay is treated as a directory overlay (`dir`)

### Requirement: Support an opt-in governance config and lockfile
The system SHALL support an opt-in governance configuration file at:

`repo/agentpack.org.yaml`

The system SHALL support an opt-in governance lockfile at:

`repo/agentpack.org.lock.json`

Core commands (`plan`, `diff`, `deploy`, etc.) MUST NOT read the governance config or lockfile.

The governance config MAY reference a policy pack source and the system SHALL be able to pin that source via the governance lockfile for auditability.

#### Scenario: core commands ignore governance config
- **GIVEN** `repo/agentpack.org.yaml` exists
- **WHEN** the user runs `agentpack plan`
- **THEN** core behavior is unchanged (no governance config is read)

### Requirement: Governance config supports distribution_policy
The system SHALL support an optional “org distribution policy” section in the governance config file:

`repo/agentpack.org.yaml`

The distribution policy MUST be scoped to governance commands (`agentpack policy ...`) and MUST NOT change the behavior of core commands (`plan`, `diff`, `deploy`, etc.).

The distribution policy MAY declare requirements over the repo’s manifest (`repo/agentpack.yaml`), including:
- required targets
- required modules (enabled)

#### Scenario: core commands ignore org distribution policy
- **GIVEN** `repo/agentpack.org.yaml` configures `distribution_policy`
- **WHEN** the user runs `agentpack plan`
- **THEN** core behavior is unchanged (no governance config is read)

### Requirement: Conformance tests run in temporary roots without writing to real home

The repository SHALL ensure target conformance tests:
- run entirely within temporary directories,
- do not rely on real user home or machine state, and
- can execute safely in parallel.

#### Scenario: conformance tests do not write outside temp roots
- **WHEN** the conformance test suite is executed
- **THEN** it does not read or write outside test-managed temporary roots

### Requirement: Conformance tests cover Windows path and permission edge cases

The repository SHALL include conformance tests that exercise Windows path and permission boundary cases, including:
- invalid path characters
- path too long
- read-only destination files
- permission denied writes

#### Scenario: conformance suite validates Windows edge cases
- **GIVEN** the test suite is running on Windows
- **WHEN** conformance tests execute
- **THEN** failures return stable JSON error codes with actionable messages

### Requirement: evolve propose skipped items include structured reason fields (additive)

When invoked as `agentpack evolve propose --dry-run --json`, each `data.skipped[]` item MUST include:
- `reason_code` (stable, enum-like string)
- `reason_message` (human-readable explanation)
- `next_actions[]` (suggested follow-up commands; may be empty)

This change MUST be additive for `schema_version=1` (existing fields like `reason` remain).

Initial `reason_code` values emitted by `evolve.propose` SHOULD include:
- `missing`
- `multi_module_output`

#### Scenario: missing drift includes reason_code and next_actions
- **GIVEN** an expected managed output is missing on disk
- **WHEN** the user runs `agentpack evolve propose --dry-run --json`
- **THEN** `data.skipped[]` contains an item with `reason=missing`
- **AND** that item contains `reason_code=missing`
- **AND** that item contains a non-empty `reason_message`
- **AND** `next_actions[]` includes at least one safe follow-up command
