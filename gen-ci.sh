#!/usr/bin/env bash

set -euo pipefail

# Generate into a temporary file and move it into place, so that a failing
# `jsonnet` run (an unpinned action, say) aborts without leaving a truncated
# workflow behind.
trap 'rm -f .github/workflows/*.yml.tmp' EXIT

for CRATE_PATH in crates/*; do
    CRATE_NAME=$(basename "${CRATE_PATH}")
    CARGO_TOML="${CRATE_PATH}/Cargo.toml"
    if [ ! -f "$CARGO_TOML" ]; then
        continue
    fi

    CI_CONFIG_YML="${CRATE_PATH}/ci.config.yml"
    WORKFLOW_YML=.github/workflows/${CRATE_NAME}.yml
    echo ${WORKFLOW_YML}

    CRATE=$(yq .package.name "$CARGO_TOML")
    RUST_VERSION=$(yq .package.rust-version "$CARGO_TOML")
    CONFIG=$( [ -f "$CI_CONFIG_YML" ] && yq -o=json "$CI_CONFIG_YML" || echo '{}' )
    jsonnet ci.jsonnet \
        -V crate="$CRATE" \
        -V rust_version="$RUST_VERSION" \
        -V config="$CONFIG" \
        | yq -P '
            (.. | select(tag == "!!map" and has("_version"))) |= (
              .uses line_comment = ._version | del(._version)
            )
          ' \
        > "${WORKFLOW_YML}.tmp"
    mv "${WORKFLOW_YML}.tmp" "${WORKFLOW_YML}"
done
