#!/bin/bash
for x in */; do
    pushd $x
    if cargo build --release; then
        popd
    else
        break
    fi
done
