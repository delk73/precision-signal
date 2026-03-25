#!/usr/bin/env bash
set -euo pipefail

CI_WF=".github/workflows/ci.yml"
WASM_WF=".github/workflows/opt_wasm.yml"

CHECKOUT_SHA="692973e3d937129bcbf40652eb9f2f61becf3332"
RUST_TOOLCHAIN_SHA="0f44b27771c32bda9f458f75a1e241b09791b331"
RUST_CACHE_SHA="779680da715d629ac1d338a641029a2f4372abb5"

require_line() {
  local file="$1"
  local regex="$2"
  if ! grep -Eq "$regex" "$file"; then
    echo "CI pin check failed: missing required pattern in $file:" >&2
    echo "  $regex" >&2
    exit 1
  fi
}

require_absent() {
  local file="$1"
  local pattern="$2"
  if grep -Eq "$pattern" "$file"; then
    echo "CI pin check failed: forbidden pattern found in $file:" >&2
    echo "  $pattern" >&2
    exit 1
  fi
}

require_line "$CI_WF" '^[[:space:]]*runs-on:[[:space:]]*ubuntu-24\.04[[:space:]]*$'
require_line "$WASM_WF" '^[[:space:]]*runs-on:[[:space:]]*ubuntu-24\.04[[:space:]]*$'

require_line "$CI_WF" "^[[:space:]]*uses:[[:space:]]*actions/checkout@${CHECKOUT_SHA}[[:space:]]*$"
require_line "$CI_WF" "^[[:space:]]*uses:[[:space:]]*dtolnay/rust-toolchain@${RUST_TOOLCHAIN_SHA}[[:space:]]*$"
require_line "$CI_WF" "^[[:space:]]*uses:[[:space:]]*Swatinem/rust-cache@${RUST_CACHE_SHA}[[:space:]]*$"

require_line "$WASM_WF" "^[[:space:]]*uses:[[:space:]]*actions/checkout@${CHECKOUT_SHA}[[:space:]]*$"
require_line "$WASM_WF" "^[[:space:]]*uses:[[:space:]]*dtolnay/rust-toolchain@${RUST_TOOLCHAIN_SHA}[[:space:]]*$"
require_line "$WASM_WF" "^[[:space:]]*uses:[[:space:]]*Swatinem/rust-cache@${RUST_CACHE_SHA}[[:space:]]*$"

require_absent "$CI_WF" 'runs-on:\s*.*-latest'
require_absent "$WASM_WF" 'runs-on:\s*.*-latest'
require_absent "$CI_WF" 'uses:\s*[^@[:space:]]+@(v[^[:space:]]*|stable|main|master|release|HEAD)(\b|$)'
require_absent "$WASM_WF" 'uses:\s*[^@[:space:]]+@(v[^[:space:]]*|stable|main|master|release|HEAD)(\b|$)'

if grep -E '^[[:space:]]*uses:[[:space:]]*' "$CI_WF" \
  | grep -vE '^[[:space:]]*uses:[[:space:]]*[^@[:space:]]+@([0-9a-f]{40})[[:space:]]*$'
then
  echo "CI pin check failed: non-SHA uses ref found in $CI_WF" >&2
  exit 1
fi

if grep -E '^[[:space:]]*uses:[[:space:]]*' "$WASM_WF" \
  | grep -vE '^[[:space:]]*uses:[[:space:]]*[^@[:space:]]+@([0-9a-f]{40})[[:space:]]*$'
then
  echo "CI pin check failed: non-SHA uses ref found in $WASM_WF" >&2
  exit 1
fi

echo "CI pin check OK"
