for x in *; do
    pushd $x
    cargo build --release
    popd
done
