#!/bin/bash

# Update direct dependencies to the minimum specified version
# leaving transitive dependencies at the latest version. As there
# is no built-in command for this in cargo we do work around this
# by pinning the dependencies:
#
#   1. Update all dependencies to the minimum version via
#      `cargo update -Z minimal-version`
#
#   2. Extract the minimum versions of the direct dependencies from
#      the `Cargo.lock` file
#
#   3. Pin the version in the `Cargo.toml`
#
#   4. Run `cargo.update`
#
#   5. Restore `Cargo.toml` to original version
#
# This process is rather slow but as of now neither `-Z minimal-version`
# nor `-Z direct-minimal-version` does what we need to implement a proper
# check for minimum required versions.

set -eu

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <rust-version>" >&2
  exit 1
fi

RUST_VERSION=$1

cp Cargo.toml Cargo.toml.bak
trap 'mv Cargo.toml.bak Cargo.toml' EXIT

cargo +nightly update -Z minimal-versions

deps=$(yq e '.dependencies | to_entries | map(.value.package // .key) | .[]' Cargo.toml)

for dep in $deps; do
  if [[ "$dep" == "deadpool" || "$dep" == deadpool-* ]]; then
    # skip deadpool dependencies
    continue
  fi
  # FIXME "tail -n 1" is really just a quick fix! This code should
  # find the correct version via the semantic version rules.
  version=$(yq e --input-format=toml --output-format=toml ".package[] | select(.name == \"$dep\") | .version" Cargo.lock | tail -n 1)
  if [[ -n $version ]]; then
    echo "Pinning $dep to $version"
    cargo add "$dep@=$version"
  else
    echo "Warning: No version found for $dep"
    exit 1
  fi
done

cargo "+$RUST_VERSION" update
