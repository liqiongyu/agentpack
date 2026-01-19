# Minimal developer task entrypoints (Codex-style).

default:
  @just --list

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all --locked

nextest:
  cargo nextest run --all --locked

audit:
  cargo audit

deny:
  cargo deny check licenses bans sources

check: fmt-check clippy test
