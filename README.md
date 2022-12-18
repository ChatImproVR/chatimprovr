# ChatImproVR
Crates:
* `client`: Client application, provides rendering, input, and other user interfacing
* `server`: Server application, a headless service
* `engine`: WASM Plugin, ECS, and messaging layer for use in implementing server and client
* `engine_interface`: Engine interface for use within e.g. plugins
* `common`: Interfacing data types between provided plugin, client, and server e.g. position component
* `plugin`: An example plugin

Plugins are required to import the `engine_interface` crate, and often import the `common` crate

# Preparation
Make sure you have the `wasm32-unknown-unknown` target installed;
```sh
rustup target add wasm32-unknown-unknown
```

![Visual aid for crate graph](./graph.svg)

# TODO
* [ ] Use real UUIDs instead of these random numbers and silly ID constants
* [ ] All of the other TODOs... `grep -ir 'todo' */src/*`
