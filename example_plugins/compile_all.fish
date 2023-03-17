#!/bin/fish
for x in */
    pushd $x
    if cargo build --release
        popd
    else
        break
    end
end
