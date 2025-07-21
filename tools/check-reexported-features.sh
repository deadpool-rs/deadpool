#!/bin/bash

set -eu

TEMP_DIR=$(mktemp -d)
# shellcheck disable=SC2064
trap "rm -r ${TEMP_DIR}" EXIT

METADATA=$TEMP_DIR/metadata.json
DEPENDENCY_FEATURES=$TEMP_DIR/dependency-features.json
REEXPORTED_FEATURES=$TEMP_DIR/reexported-features.json

BACKEND=$(yq ".backend // \"\"" ci.config.yml)
FEATURES_OWN=$(yq --output-format=json ".features | .own // []" ci.config.yml)
FEATURES_EXCLUDE=$(yq --output-format=json ".features | .exclude // []" ci.config.yml)

if [ -z "$BACKEND" ]; then
    echo '"backend" missing in ci.config.yml'
    exit 1
fi

# Replace `-` by `_` as Cargo doesn't actually use `-` in package names.
# e.g. `tokio-postgres` becomes `tokio_postgres`.
BACKEND_NORMALIZED=${BACKEND//-/_}

cargo metadata --format-version 1 > $METADATA

CRATE_NAME=$(yq .package.name Cargo.toml)

# We need the precise resolved ID because there is multiple versions of 'redis' in dependencies
DEPENDENCY_ID=$(
    yq "
        .resolve
        | .root as \$root
        | .nodes[]
        | select(.id == \$root)
        | .deps[]
        | select(.name == \"$BACKEND_NORMALIZED\")
        | .pkg
    " \
    $METADATA
)

if [ -z "$DEPENDENCY_ID" ]; then
    echo "dependency \"${BACKEND}\" not found"
    exit 1
fi

yq --output-format=json \
    "
        (
            .features
            | keys()
            - [\"default\"]
        )
        - $FEATURES_OWN
        | sort
    " \
    Cargo.toml \
    | jq .[] --raw-output \
    > $REEXPORTED_FEATURES

jq --raw-output \
    "
        [
            .packages[]
            | select(.id == $DEPENDENCY_ID)
            | .features
            | to_entries[]
            # All direct dependency 'a' is considered a feature 'a' with 'dep:a'
            # Let's remove all of them
            | select((.value | length) != 1 or \"dep:\"+.key != .value[0])
            | .key
        ]
        # Remove 'default' feature, we won't expose it
        - [ \"default\" ]
        # Remove all features that should be ignored
        - $FEATURES_EXCLUDE
        | sort
        | .[]
    " \
    < $METADATA \
    > $DEPENDENCY_FEATURES

# 'diff' returns 0 if no difference is found
printf "%-63s %s\n" "$BACKEND features" "$CRATE_NAME features"
echo -e "------------------------------                                  ------------------------------"
diff --color --side-by-side "${DEPENDENCY_FEATURES}" "${REEXPORTED_FEATURES}"
