# TARGET_SDK.md

Goal: make adding a new target predictable and reviewable.

## New target checklist
0. Start from `TARGET_MAPPING_TEMPLATE.md` and fill it out.
1. Pick a stable target id (e.g., `cursor`).
2. Define managed roots (each root MUST write `.agentpack.manifest.<target>.json`).
3. Define mapping rules (modules → output paths under roots).
4. Define minimal validation (structure/frontmatter requirements).
5. Pass conformance tests (see `TARGET_CONFORMANCE.md`).

## Footguns to avoid
- Never delete unmanaged files (only delete manifest-managed paths).
- Don’t rely on symlinks (default is copy/render).
- Keep target-specific behavior documented and tested.
