pushd client
base="../target/wasm32-unknown-unknown/release"
cargo run --release --bin cimvr_client -- $base/plugin*.wasm 
