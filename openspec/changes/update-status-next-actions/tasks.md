## 1. Spec
- [x] Add OpenSpec delta for `agentpack-cli` describing `status.data.next_actions` (additive) and scenarios.

## 2. Implementation
- [x] Add `next_actions` computation to `src/cli/commands/status.rs` (JSON + human).
- [x] Update `src/cli/commands/schema.rs` to include `status.next_actions` in documented `data_fields`.

## 3. Docs
- [x] Update `docs/SPEC.md` `status` section to mention `next_actions` suggestions (additive).
- [x] Update `docs/JSON_API.md` `status` section to document `data.next_actions`.

## 4. Tests
- [x] Update golden snapshots affected by the new field (`tests/golden/schema_json_data.json`, status JSON golden).
- [x] Run `cargo fmt`, `cargo clippy`, `cargo test --all --locked`.
- [x] Run `openspec validate update-status-next-actions --strict --no-interactive`.
