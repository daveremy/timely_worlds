#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="${SCRIPT_DIR}/.."
cd "$ROOT"
RUST_LOG=${RUST_LOG:-info} \
    cargo run -p tw-examples --bin retail_demo "$@"
