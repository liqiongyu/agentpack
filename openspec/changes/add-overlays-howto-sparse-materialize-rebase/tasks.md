## 1. Implementation

- [ ] 1.1 Add `docs/howto/overlays-create-sparse-materialize-rebase.md` (task-oriented overlays workflow).
- [ ] 1.2 Link the new how-to from `docs/index.md` (and `docs/zh-CN/index.md` if applicable).

## 2. Spec deltas

- [ ] 2.1 Add a delta requirement that overlays docs include a task-oriented how-to for sparse/materialize/rebase workflows.

## 3. Validation

- [ ] 3.1 `openspec validate add-overlays-howto-sparse-materialize-rebase --strict --no-interactive`
- [ ] 3.2 `cargo test --all --locked`
