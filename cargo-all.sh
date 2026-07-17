#!/bin/bash

set -eux

for crate_dir in crates/deadpool*; do (
    cd $crate_dir
    cargo $@
); done
