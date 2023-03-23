docker run \
    --env CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    --rm \
    -it \
    -w /cwd \
    -v $PWD:/cwd \
    rust cargo build --bin cimvr_server --release
