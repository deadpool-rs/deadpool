#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo run \
  --manifest-path "$ROOT_DIR/tools/release-manager/Cargo.toml" \
  -- \
  status \
  --repo-root "$ROOT_DIR" \
  "$@"
