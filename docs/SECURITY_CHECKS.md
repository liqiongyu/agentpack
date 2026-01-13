# Security checks (CI defaults)

This repository enables two dependency/security checks in CI by default.

## 1) RustSec advisory scan (`cargo audit`)

- CI: `.github/workflows/ci.yml` → `Security audit` job (`rustsec/audit-check`)
- Local (optional): install and run `cargo audit`
  - Install: `cargo install cargo-audit --locked`

How to handle failures:
- Prefer upgrading/replacing dependencies to eliminate advisories.
- If you must temporarily ignore something (not recommended): document the reason, impact, and a concrete removal plan in the PR/issue.

## 2) `cargo-deny` dependency policy (licenses / bans / sources)

- CI: `.github/workflows/ci.yml` → `Dependency policy (cargo-deny)` job
- Config: `deny.toml`
- Checks currently run in CI:
  - `licenses`: allowlist/exceptions
  - `bans`: duplicate versions and wildcard constraints (`*`)
  - `sources`: dependency sources (allow crates.io only; disallow unknown registries/git by default)

Run locally:
- `cargo install cargo-deny@0.18.3 --locked`
- `cargo deny check licenses bans sources`

How to handle failures:
- `licenses`:
  - If the new license is acceptable: add its SPDX identifier to `deny.toml` → `licenses.allow`
  - If only one crate needs an exception: use `licenses.exceptions` (and document `reason`/issue)
- `sources`:
  - Avoid git deps when possible; if you must, add it explicitly under `sources.allow-git` (and document `reason`/issue)
- `bans`:
  - If `wildcards` (`*`) is rejected: pin the dependency to an explicit semver range
  - `multiple-versions` is currently a warning: reduce over time via dependency upgrades and `cargo update -p <crate>`
