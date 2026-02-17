#!/bin/bash

set -eu

TEMP_DIR=$(mktemp -d)
# shellcheck disable=SC2064
trap "rm -r ${TEMP_DIR}" EXIT

METADATA=$TEMP_DIR/metadata.json
DEPENDENCY_FEATURES=$TEMP_DIR/dependency-features.json
REEXPORTED_FEATURES=$TEMP_DIR/reexported-features.json
BACKEND_EXPLICIT_FEATURES=$TEMP_DIR/backend-explicit-features.json
BACKEND_OPTIONAL_DEPENDENCY_NAMES=$TEMP_DIR/backend-optional-dependency-names.json

BACKEND=$(yq ".backend // \"\"" ci.config.yml)
FEATURES_OWN=$(yq --output-format=json ".features | .own // []" ci.config.yml)
FEATURES_OPTIONAL_DEPENDENCIES=$(yq --output-format=json '.features | ."optional-dependencies" // []' ci.config.yml)
FEATURES_EXCLUDE=$(yq --output-format=json ".features | .exclude // []" ci.config.yml)
CRATE_FEATURES_JSON=$(yq --output-format=json ".features // {}" Cargo.toml)

if [ -z "$BACKEND" ]; then
    echo '"backend" missing in ci.config.yml'
    exit 1
fi

# Features listed in `features.own` are usually removed from the comparison.
# Keep those that also explicitly re-export same-name backend features
# (e.g. `serde = [..., "libsql/serde"]`).
FEATURES_OWN_EXCLUDE=$(
    jq --null-input --compact-output \
        --arg backend "$BACKEND" \
        --argjson own "$FEATURES_OWN" \
        --argjson features "$CRATE_FEATURES_JSON" \
        '
            $own
            | map(
                . as $feature
                | select((($features[$feature] // []) | index("\($backend)/\($feature)")) | not)
            )
        '
)

# Replace `-` by `_` as Cargo doesn't actually use `-` in package names.
# e.g. `tokio-postgres` becomes `tokio_postgres`.
BACKEND_NORMALIZED=${BACKEND//-/_}

cargo metadata --format-version 1 > "$METADATA"

CRATE_NAME=$(yq .package.name Cargo.toml)

# We need the precise resolved ID because there is multiple versions of 'redis' in dependencies
DEPENDENCY_ID=$(
    jq --raw-output --arg backend "$BACKEND_NORMALIZED" '
        .resolve
        | .root as $root
        | .nodes[]
        | select(.id == $root)
        | .deps[]
        | select(.name == $backend)
        | .pkg
    ' \
    "$METADATA"
)

if [ -z "$DEPENDENCY_ID" ]; then
    echo "dependency \"${BACKEND}\" not found"
    exit 1
fi

DEPENDENCY_MANIFEST=$(
    jq --raw-output --arg dependency_id "$DEPENDENCY_ID" '
        .packages[]
        | select(.id == $dependency_id)
        | .manifest_path
    ' \
    "$METADATA"
)

if [ -z "$DEPENDENCY_MANIFEST" ] || [ "$DEPENDENCY_MANIFEST" = "null" ]; then
    echo "could not determine manifest path for dependency \"${BACKEND}\""
    exit 1
fi

yq --output-format=json \
    "
        .features
        | keys
        | . - [\"default\"]
        | . - $FEATURES_OWN_EXCLUDE
        | sort
    " \
    Cargo.toml \
    | jq --raw-output .[] \
    > "$REEXPORTED_FEATURES"

yq --output-format=json \
    "
        .features // {}
        | keys
        | . - [\"default\"]
        # Remove all features that should be ignored
        | . - $FEATURES_EXCLUDE
        | sort
    " \
    "$DEPENDENCY_MANIFEST" \
    > "$BACKEND_EXPLICIT_FEATURES"

yq --output-format=json \
    '
        .dependencies // {}
        | to_entries
        | map(select((.value.optional // false) == true) | .key)
        | sort
    ' \
    "$DEPENDENCY_MANIFEST" \
    > "$BACKEND_OPTIONAL_DEPENDENCY_NAMES"

OPTIONAL_DEPENDENCY_VALIDATION=$(
    jq --null-input --compact-output \
        --argjson configured "$FEATURES_OPTIONAL_DEPENDENCIES" \
        --argjson backend_features "$(cat "$BACKEND_EXPLICIT_FEATURES")" \
        --argjson backend_optional_dependencies "$(cat "$BACKEND_OPTIONAL_DEPENDENCY_NAMES")" \
        '
            {
                moved_to_features:
                    ($configured
                        | map(
                            . as $feature
                            | select(($backend_features | index($feature)) != null)
                        )),
                missing_optional_dependencies:
                    ($configured
                        | map(
                            . as $feature
                            | select(($backend_optional_dependencies | index($feature)) == null)
                        ))
            }
        '
)

if [ "$(jq --raw-output '.moved_to_features | length' <<< "$OPTIONAL_DEPENDENCY_VALIDATION")" -ne 0 ]; then
    echo "These entries in features.optional-dependencies are explicit backend features now:"
    jq --raw-output '.moved_to_features[]' <<< "$OPTIONAL_DEPENDENCY_VALIDATION"
    exit 1
fi

if [ "$(jq --raw-output '.missing_optional_dependencies | length' <<< "$OPTIONAL_DEPENDENCY_VALIDATION")" -ne 0 ]; then
    echo "These entries in features.optional-dependencies are not optional dependencies in the backend:"
    jq --raw-output '.missing_optional_dependencies[]' <<< "$OPTIONAL_DEPENDENCY_VALIDATION"
    exit 1
fi

jq --null-input --raw-output \
    --argjson backend_features "$(cat "$BACKEND_EXPLICIT_FEATURES")" \
    --argjson configured_optional_dependencies "$FEATURES_OPTIONAL_DEPENDENCIES" \
    '
        ($backend_features + $configured_optional_dependencies)
        | unique
        | sort
        | .[]
    ' \
    > "$DEPENDENCY_FEATURES"

# 'diff' returns 0 if no difference is found
printf "%-63s %s\n" "$BACKEND features" "$CRATE_NAME features"
echo -e "------------------------------                                  ------------------------------"
diff --color --side-by-side "${DEPENDENCY_FEATURES}" "${REEXPORTED_FEATURES}"
