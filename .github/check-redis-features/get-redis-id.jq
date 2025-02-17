.resolve
| .root as $root
| .nodes[]
| select(.id == $root)
| .deps[]
| select(.name == "redis")
| .pkg
