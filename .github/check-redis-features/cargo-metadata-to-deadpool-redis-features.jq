[
  .packages[]
  | select(.name == "deadpool-redis")
  | .features
  | keys[]
]
- [ "default"]
- $deadpool_features
| .[]
