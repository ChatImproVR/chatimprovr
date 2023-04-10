pushd client
cargo b -r
popd

pushd server
cargo b -r
popd

pushd example_plugins
./compile_all.sh
popd
