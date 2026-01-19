## 1. Implementation

- [x] Add `--markdown` output mode to `agentpack help` (deterministic Markdown).
- [x] Update `docs/reference/cli.md` to be generated (or semi-generated) from the Markdown output.
- [x] Add a CI check (test) that compares generated Markdown to `docs/reference/cli.md`.

## 2. Contract updates

- [x] Update any golden snapshots impacted by the new `help` flag (e.g., `help --json` output describing `help` args).

## 3. Validation

- [x] `openspec validate add-help-markdown --strict`
- [x] `cargo test --all --locked`
