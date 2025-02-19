[
  .packages[]
  | select(.name == $deadpool_crate)
  | .features
  | keys[]
]
- [ "default"]
- $deadpool_features
| .[]
