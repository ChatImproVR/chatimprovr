base="./target/wasm32-unknown-unknown/release"
cargo build --release --bin cimvr_client &&\
    perf record --call-graph=lbr\
    ./target/release/cimvr_client $base/plugin.wasm $base/plugin2.wasm

