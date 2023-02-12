#!/usr/bin/env python3
import os
from os.path import dirname, join, isfile
import argparse
from subprocess import Popen
from time import sleep

# TODO: Enable this script to `cargo run` the server and client.


def main():
    parser = argparse.ArgumentParser(
        prog='ChatImproVR helper script',
        description='Launches the client and server, finds plugin paths',
        epilog='Also searches CIMVR_PLUGINS for WASM paths'
    )
    parser.add_argument(
        "plugins",
        nargs='*',
        help=""",
        Plugins can be the truncated
        (thing.wasm -> thing) form, or full paths.
        """
    )
    parser.add_argument(
        "--client", "-c",
        action='store_true',
        help="Only run the client"
    )
    parser.add_argument(
        "--server", "-s",
        action='store_true',
        help="Only run the server"
    )
    parser.add_argument(
        "--verbose", "-v",
        help="Verbose debug output"
    )
    args = parser.parse_args()

    # The script is assumed to be at the root of the project
    root_path = dirname(__file__)

    if args.verbose:
        print(f"Root path: {root_path}")

    # Client + Server behaviour
    if not args.client and not args.server:
        args.client = True
        args.server = True

    # Find executables
    server_exe = find_exe(
        "CIMVR_SERVER",
        ["cimvr_server", "cimvr_server.exe"],
        root_path
    )
    if args.verbose:
        print(f"Server exe: {server_exe}")
    if not server_exe:
        print("Failed to find server executable")
        return

    client_exe = find_exe(
        "CIMVR_CLIENT",
        ["cimvr_client", "cimvr_client.exe"],
        root_path
    )
    if args.verbose:
        print(f"Client exe: {client_exe}")
    if not client_exe:
        print("Failed to find client executable")
        return

    # Find all plugins
    plugins = []
    for name in args.plugins:
        # Just a file
        if isfile(name):
            plugins.append(name)
            continue

        # Truncated name of some search folder
        path = find_wasm(name, root_path)
        if path:
            plugins.append(path)
        else:
            print(f"No plugin named \"{name}\" found.")
            return

    if args.verbose:
        print("Plugins:")
        for p in plugins:
            print(f"Plugin {p}")

    # Decide on a list of executables
    exes = []
    if args.server:
        exes += [server_exe]

    if args.client:
        exes += [client_exe]

    # Launch client an server
    procs = []
    for exe in exes:
        print(exe)
        procs.append(Popen([exe] + plugins))
        # Wait for server to start
        sleep(0.1)

    for p in procs:
        p.wait()


def find_wasm(name, root_path):
    # Search the build path, and a local "plugins" folder
    wasm_target = "wasm32-unknown-unknown"
    build_path = join(root_path, "target", wasm_target, "release")

    plugin_folders = [join(root_path, "plugins"), build_path]

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


def find_exe(env_var, names, root_path):
    """
    Look for the given environment variable, or try looking adjacent to the
    script, or in the build path adjacent the script. Returns None if it cannot
    find the exe.
    """
    if env_var in os.environ:
        return os.environ[env_var]
    else:
        build_path = join(root_path, "target", "release")
        client_build_path = join(root_path, "client", "target", "release")
        possible_locations = \
            [join(root_path, x) for x in names]\
            + [join(build_path, x) for x in names]\
            + [join(client_build_path, x) for x in names]

        for path in possible_locations:
            if isfile(path):
                return path

    return None


if __name__ == "__main__":
    main()
