#!/bin/bash

for crate in crates/*; do
    echo
    [ -f $crate/ci.config.yml ] || continue;
    [ -n $(yq ".backend // \"\"" $crate/ci.config.yml) ] || continue
    (cd $crate; ../../tools/check-reexported-features.sh)
done
