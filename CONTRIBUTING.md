# Contributing

## Workflow

- Small fixes: open a PR directly.
- Feature/architecture work: use OpenSpec proposals in `openspec/` (see `docs/CONTRIBUTING_SPECS.md` and `openspec/AGENTS.md`).

## Local checks

Required before opening a PR:

### Rust toolchain

This repo pins Rust via `rust-toolchain.toml` (MSRV). Use `rustup` so the pinned toolchain is respected.

If you have both Homebrew Rust and rustup installed, make sure `$HOME/.cargo/bin` is before `/opt/homebrew/bin` in your `PATH`, otherwise `cargo` may come from Homebrew and fail MSRV checks.

Quick checks:
- `rustc -V`
- `cargo -V`

```bash
# Preferred (matches CI): install `just` (`cargo install just`) and run:
just check

# Or run the underlying cargo commands directly:
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
```

Optional: install git hooks via `pre-commit`:
- Install: `uv pip install pre-commit --system` (uv) or `pipx install pre-commit` (pipx) or `pip install pre-commit`
- Enable: `pre-commit install`
- Run all hooks: `pre-commit run -a`

## GitHub operations (CLI)

Prefer GitHub CLI (`gh`) over the web UI:
- Issues: `gh issue create`, `gh issue list`, `gh issue view`
- PRs: `gh pr create`, `gh pr view`, `gh pr checkout`

## PR expectations

- Clear description of intent and behavior changes.
- Link related issues (or explain why none).
- Evidence: paste key command output (e.g., `cargo test --all --locked`), and include relevant `--json` output when implementing CLI behavior.
