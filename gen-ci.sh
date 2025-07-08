#!/bin/bash

set -e

for CRATE_PATH in crates/*; do
    CRATE_NAME=$(basename "${CRATE_PATH}")
    CARGO_TOML="${CRATE_PATH}/Cargo.toml"
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
        | yq -P \
        > "${WORKFLOW_YML}"
done
