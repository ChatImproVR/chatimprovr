pushd client
if cargo b -r; then
    popd
else
    exit
fi

pushd server
if cargo b -r; then
    popd
else
    exit
fi

pushd example_plugins
./compile_all.sh
popd
