# Architecture
## Bird's Eye View
![Birds eye view](https://github.com/ChatImproVR/iteration0/assets/6164303/d77935f3-32d3-49f5-bb45-237f4acd2b3a)

ChatImproVR is a multiplayer game engine focused around the use of plugins. 

Plugins in ChatImproVR are implemented as WebAssembly modules. This interface exposes the two main communication methods available to plugins; pub/sub messaging and access to the Entity Component System. Plugins declare systems, which the host program (client or server) executes. Using these interfaces, plugins can communicate with each other, and the host. Clients connect to a server and communicate with it over the network. Both the client and server have their own independent ECS databases, although there are mechanisms to synchronize data between them. 

## Code map
### `engine`
Both hosts (the `client` and the `server` crates) make use of the `Engine`, which is found in the `engine` crate. The `Engine` contains the core functionality for loading and talking to WASM plugins, as well as the `Ecs` object which contains the ECS database. The engine also contains functionality for re-loading plugins when their associated files change; see the `hotload` module.

**Architecture Invariant**: Code in the `engine` crate is intended to be platform agnostic.

### `server`
The Server is a headless service which hosts a virtual world; the `server` crate loads an arbitrary number of WASM plugins and executes them in a loop. It listens for connections from clients. It provides facilities for managing connected clients. 

### `client`
The Client is a user-facing application which displays a virtual world; the `client` crate loads an arbitrary number of WASM plugins and executes them in a loop. It does not work without connection to a server, even if it is local. It contains code for opening a window and getting an OpenGL instance, rendering, GUI integration, VR support, and more. See the individual modules therein.

### `engine_interface`
The `engine_interface` crate is depended on by virtually all ChatImproVR-related code, and provides the basic data types used to communicate across the host/plugin boundary. Specifically, the `engine_interface` crate contains all of the code which is necessary for a WASM plugin to talk to the `Engine` (for example, the `Component` trait). 

**Architecture Invariant**: The engine interface cannot assume access to IO beyond allocation/deallocation, because it must run inside of WASM. 

### `common`
The `common` crate contains the rest of the code needed to interface with the specific functionality implemented by the `client` and `server` crates (for example, the `Render` component). This separation makes it possible to cleanly develop ChatImproVR-compatible hosts with new roles, and keeps the interface free from clutter.

**Architecture Invariant**: The engine interface cannot assume access to IO beyond allocation/deallocation, because it must run inside of WASM. 

### `example_plugins`
The `example_plugins` folder contains many crates which have been configured to compile as plugins. These demonstrate what ChatImproVR can do, or just look pretty :)
