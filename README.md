# ChatImproVR
Crates:
* `client`: Client application, provides rendering, input, and other user interfacing
* `server`: Server application, a headless service
* `engine`: WASM Plugin, ECS, and messaging layer for use in implementing server and client
* `engine_interface`: Engine interface for use within e.g. plugins
* `common`: Interfacing data types between provided plugin, client, and server e.g. position component
* `plugin`: An example plugin

Plugins are required to import the `engine_interface` crate, and often import the `common` crate
