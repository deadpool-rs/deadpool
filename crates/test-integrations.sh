#!/bin/bash

set -eux

cargo test --lib --no-default-features --features serde,core
cargo test --lib --no-default-features --features serde,remote,tls
cargo test --lib --no-default-features --features serde,replication,tls
cargo test --lib --no-default-features --features serde,sync,tls
