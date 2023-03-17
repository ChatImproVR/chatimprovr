# ChatImproVR
Crates:
* `client`: Client application, provides rendering, input, and other user interfacing
* `server`: Server application, a headless service
* `engine`: WASM Plugin, ECS, and messaging layer for use in implementing server and client
* `engine_interface`: Engine interface for use within e.g. plugins
* `common`: Interfacing data types between provided plugin, client, and server e.g. position component
* `plugin`: An example plugin (currently moves the camera)
* `plugin2`: An example plugin (currently adds and moves cubes)

Plugins are required to import the `engine_interface` crate, and often import the `common` crate

# Preparation
Make sure you have the `wasm32-unknown-unknown` target installed;
```sh
rustup target add wasm32-unknown-unknown
```

Dependencies on Ubuntu:
```sh
sudo apt install build-essential cmake libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev libudev-dev libfontconfig-dev
```

# Compilation
Build the client and server like so:
```sh
pushd server
cargo build --release
popd

pushd client
cargo build --release
popd
```

You can compile all of the example plugins at once with the `compile_all` script.

While most crates _are_ in a workspace, the client crate is unfortunately excluded due to an issue with the `openxr` crate. 

We also cannot compile all of the crates in the workspace from the root level, because only the server is compiled for your native platform, but the plugins must be compiled for wasm32. Currently, [cargo will only compile a workspace for the native target](https://github.com/rust-lang/cargo/issues/7004).

# Hosting a server
You may host a server with
```sh
cargo run --release -- <plugins>
```

# Connection to a remote server
You may use the client to connect to a remote server like so:
```sh
cargo run --release -- --connect <ip>:5031 <plugins>
```

The default port is 5031, but this can be configured in the server with `--bind <bind addr>:<port>`

# Organization 
![Visual aid for crate graph](./graph.svg)

Plugins are required to import `engine_interface`. Most plugins will need to import `common`, as it provides interfacing with the specific functionality implemented in the provided client and server.
