#!/usr/bin/env sh
ROOT_PATH=$(dirname $0)
METADATA_JSON=/tmp/metadata.json
REDIS_FEATURES=/tmp/redis.features
DEADPOOL_REDIS_FEATURES=/tmp/deadpool-redis.features

  # It's important to run it from `redis/` since finding `REDIS_ID` below
  # will depend on which crate is resolved
cargo \
  metadata \
  --format-version 1 \
  --manifest-path "redis/Cargo.toml" \
  > "${METADATA_JSON}"

# We need the precise resolved ID because there is multiple versions of 'redis' in dependencies
REDIS_ID=$( jq \
  --raw-output \
  --from-file "${ROOT_PATH}/get-redis-id.jq" \
  "${METADATA_JSON}" \
)

jq \
  --raw-output \
  --arg redis_id "${REDIS_ID}" \
  --slurpfile deprecated_features "${ROOT_PATH}/deprecated.features" \
  --from-file "${ROOT_PATH}/cargo-metadata-to-redis-features.jq" \
  "${METADATA_JSON}" \
  > "${REDIS_FEATURES}"

jq \
  --raw-output \
  --slurpfile deadpool_features "${ROOT_PATH}/deadpool-redis.features" \
  --from-file "${ROOT_PATH}/cargo-metadata-to-deadpool-redis-features.jq" \
  "${METADATA_JSON}" \
  > "${DEADPOOL_REDIS_FEATURES}"

# 'diff' returns 0 if no difference is found
echo "Comparing features of 'redis' (left) and re-exposed features of 'deadpool-redis' (right)"
echo ""
echo -e "redis features\t\t\t\t\t\t\tdeadpool-redis features"
echo -e "--------------\t\t\t\t\t\t\t-----------------------"
diff --side-by-side "${REDIS_FEATURES}" "${DEADPOOL_REDIS_FEATURES}"

