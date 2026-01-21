#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

EXAMPLE_REPO_SRC="${PROJECT_ROOT}/docs/examples/minimal_repo"

if [[ ! -d "${EXAMPLE_REPO_SRC}" ]]; then
  echo "error: example repo not found: ${EXAMPLE_REPO_SRC}" >&2
  exit 1
fi

run_agentpack() {
  if [[ -n "${AGENTPACK_BIN:-}" ]]; then
    "${AGENTPACK_BIN}" "$@"
    return
  fi

  if command -v agentpack >/dev/null 2>&1; then
    agentpack "$@"
    return
  fi

  if command -v cargo >/dev/null 2>&1; then
    cargo run --quiet --manifest-path "${PROJECT_ROOT}/Cargo.toml" -- "$@"
    return
  fi

  echo "error: agentpack not found (set AGENTPACK_BIN, install agentpack, or run with Rust/cargo)" >&2
  exit 1
}

TMP_ROOT="$(mktemp -d)"
cleanup() { rm -rf "${TMP_ROOT}"; }
trap cleanup EXIT

DEMO_HOME="${TMP_ROOT}/home"
DEMO_AGENTPACK_HOME="${TMP_ROOT}/agentpack_home"
DEMO_WORKSPACE="${TMP_ROOT}/workspace"
DEMO_REPO="${TMP_ROOT}/repo"

mkdir -p "${DEMO_HOME}" "${DEMO_AGENTPACK_HOME}" "${DEMO_WORKSPACE}"
cp -R "${EXAMPLE_REPO_SRC}" "${DEMO_REPO}"

export HOME="${DEMO_HOME}"
export AGENTPACK_HOME="${DEMO_AGENTPACK_HOME}"

echo "[demo] HOME=${HOME}" >&2
echo "[demo] AGENTPACK_HOME=${AGENTPACK_HOME}" >&2
echo "[demo] repo=${DEMO_REPO}" >&2
echo "[demo] workspace=${DEMO_WORKSPACE}" >&2

cd "${DEMO_WORKSPACE}"

echo "[demo] agentpack doctor --json" >&2
run_agentpack --repo "${DEMO_REPO}" doctor --json

echo "[demo] agentpack preview --diff --json" >&2
run_agentpack --repo "${DEMO_REPO}" preview --diff --json
