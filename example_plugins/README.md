# Example plugins
Here we have a number of plugins written to demonstrate the functionality of ChatImproVR. Below is a list of the plugins, and their function.
* `hello`: A very basic example plugin which prints "Hello client" on the client and "Hello server" on the server.
* `ecs`: Basic ECS example; demonstrates creating a new component and synchronization from server to client
* `channels`: Basic Pub/Sub channel example; demonstrates creating a new message type and sending messages from client to server
* `camera`: An arcball camera for clientside, and basic VR integration. Displays hands as cubes
* `cube`: A basic example demonstrating the addition of a cube to the scene. Recommended to be used with the `camera` plugin.
* `dancing_cubes`: A somewhat more advanced example, synchronizes some animation of the server to the clients.
* `ui_example`: A basic example of GUI manipulation from plugins

If you're on Linux, you may compile all plugins at once with the `compile_all.sh` script.
