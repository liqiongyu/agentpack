# RELEASING.md

Agentpack uses `dist` (cargo-dist) to build and publish multi-platform release artifacts.

## 1) Prerequisites

- CI workflows must be green on `main`.
- `CHANGELOG.md` must be up to date for the version you’re releasing.

## 2) Cut a release

1. Bump version:
   - Update `Cargo.toml` `version = "x.y.z"`.
2. Update contract docs version stamps:
   - Update `docs/SPEC.md`, `docs/JSON_API.md`, and `docs/ERROR_CODES.md` to the new version (`vx.y.z`).
   - Ensure examples that include `"version": "..."` are updated too.
3. Update `CHANGELOG.md`:
   - Add a new section for `x.y.z` with release date.
4. Commit and push to `main`.
5. Create and push a tag:

   ```bash
   git tag -a vx.y.z -m "vx.y.z"
   git push origin vx.y.z
   ```

6. GitHub Actions `Release` workflow (`.github/workflows/release.yml`) builds artifacts and publishes a GitHub Release.

## 3) Verify

- Check the GitHub Release page:
  - Assets for Linux/macOS/Windows are attached
  - Checksums are present (`.sha256` / `sha256.sum`)

## 4) Rollback (if needed)

If a release is broken, prefer shipping a new patch release (`x.y.(z+1)`) over deleting releases.
If you must undo a release:

- Delete the GitHub Release (web UI) and the tag:
  ```bash
  git tag -d vx.y.z
  git push --delete origin vx.y.z
  ```
