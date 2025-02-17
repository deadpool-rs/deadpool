[
  .packages[]
  | select(.id == $redis_id)
  | .features
  | to_entries[]
  | # All direct dependency 'a' is considered a feature 'a' with 'dep:a'
  # Let's remove all of them
  select((.value | length) != 1 or "dep:"+.key != .value[0])
  | .key
]
# Remove 'default' feature, we won't expose it
- [ "default" ]
# Remove all deprecated features
- $deprecated_features
| .[]
