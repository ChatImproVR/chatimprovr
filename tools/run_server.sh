base="./target/wasm32-unknown-unknown/release"
cargo run --release --bin cimvr_server -- $base/plugin*.wasm
