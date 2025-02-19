#!/usr/bin/env sh
ROOT_PATH=$(dirname $0)
METADATA_JSON=/tmp/metadata.json
DEPENDENCY_FEATURES=/tmp/dependency.features
DEADPOOL_FEATURES=/tmp/deadpool.features

DEADPOOL_WORKSPACE_MEMBER="${1}"
DEPENDENCY_NAME="${2:-$DEADPOOL_WORKSPACE_MEMBER}"

  # It's important to run it from `redis/` since finding `DEPENDENCY_ID` below
  # will depend on which crate is resolved
cargo \
  metadata \
  --format-version 1 \
  --manifest-path "${DEADPOOL_WORKSPACE_MEMBER}/Cargo.toml" \
  > "${METADATA_JSON}"

# We need the precise resolved ID because there is multiple versions of 'redis' in dependencies
DEPENDENCY_ID=$( jq \
  --raw-output \
  --arg dependency "${DEPENDENCY_NAME}" \
  --from-file "${ROOT_PATH}/get-dependency-id.jq" \
  "${METADATA_JSON}" \
)

jq \
  --raw-output \
  --arg dependency_id "${DEPENDENCY_ID}" \
  --slurpfile deprecated_features "${ROOT_PATH}/${DEADPOOL_WORKSPACE_MEMBER}-deprecated.features" \
  --from-file "${ROOT_PATH}/cargo-metadata-to-features.jq" \
  "${METADATA_JSON}" \
  > "${DEPENDENCY_FEATURES}"

jq \
  --raw-output \
  --arg deadpool_crate "deadpool-${DEADPOOL_WORKSPACE_MEMBER}" \
  --slurpfile deadpool_features "${ROOT_PATH}/deadpool-${DEADPOOL_WORKSPACE_MEMBER}.features" \
  --from-file "${ROOT_PATH}/cargo-metadata-to-deadpool-features.jq" \
  "${METADATA_JSON}" \
  > "${DEADPOOL_FEATURES}"

# 'diff' returns 0 if no difference is found
echo "Comparing features of '${DEPENDENCY_NAME}' (left) and re-exposed features of 'deadpool-${DEADPOOL_WORKSPACE_MEMBER}' (right)"
echo ""
echo -e "${DEPENDENCY_NAME} features\t\t\t\t\t\t\tdeadpool-${DEADPOOL_WORKSPACE_MEMBER} features"
echo -e "--------------\t\t\t\t\t\t\t-----------------------"
diff --side-by-side "${DEPENDENCY_FEATURES}" "${DEADPOOL_FEATURES}"

