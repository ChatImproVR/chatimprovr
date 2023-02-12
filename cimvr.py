#!/usr/bin/env python3
import os
from os.path import dirname, join, isfile
import argparse
from subprocess import Popen
from time import sleep

# Get script's location, for reference
script_path = dirname(__file__)


def main():
    parser = argparse.ArgumentParser(
        prog='ChatImproVR helper script',
        description='Launches the client and server, finds plugin paths',
        epilog='Also searches CIMVR_PLUGINS for WASM paths'
    )
    parser.add_argument("plugins", nargs='+')
    args = parser.parse_args()

    # Find executables
    server_exe = find_exe("CIMVR_SERVER", ["cimvr_server", "cimvr_server.exe"])
    if not server_exe:
        print("Failed to find server executable")
        return

    client_exe = find_exe("CIMVR_CLIENT", ["cimvr_client", "cimvr_client.exe"])
    if not client_exe:
        print("Failed to find client executable")
        return

    # Find all plugins
    plugins = []
    for name in args.plugins:
        path = find_wasm(name)
        if path:
            plugins.append(path)
        else:
            print(f"No plugin named \"{name}\" found.")
            return

    # Launch client an server
    procs = []
    for exe in [server_exe, client_exe]:
        procs.append(Popen([exe] + plugins))
        # Wait for server to start
        sleep(0.1)

    for p in procs:
        p.wait()


def find_wasm(name):
    # Search the build path, and a local "plugins" folder
    wasm_target = "wasm32-unknown-unknown"
    build_path = join(script_path, "target", wasm_target, "release")

    plugin_folders = [join(script_path, "plugins"), build_path]

    # Also check CIMVR_PLUGINS, which is a semicolon-seperated list
    wasm_env_var = "CIMVR_PLUGINS"
    if wasm_env_var in os.environ:
        plugin_folders += os.environ[wasm_env_var].split(';')

    file_name = name + ".wasm"

    for folder in plugin_folders:
        path = join(folder, file_name)
        if isfile(path):
            return path

    return None


def find_exe(env_var, names):
    """
    Look for the given environment variable, or try looking adjacent to the
    script, or in the build path adjacent the script. Returns None if it cannot
    find the exe.
    """
    if env_var in os.environ:
        return os.environ[env_var]
    else:
        build_path = join(script_path, "target", "release")
        client_build_path = join(script_path, "client", "target", "release")
        possible_locations = \
            [join(script_path, x) for x in names]\
            + [join(build_path, x) for x in names]\
            + [join(client_build_path, x) for x in names]

        for path in possible_locations:
            if isfile(path):
                return path

    return None


if __name__ == "__main__":
    main()
