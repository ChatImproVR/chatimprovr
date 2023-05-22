#!/usr/bin/env python3
# Written with the help of ChatGPT, 5/21/23
import subprocess
import sys
import os
import platform

# TODO: Add an option to append to the user's shell...
# OR we should have a config file... And one cimvr command with subcommands


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <crate name> <optional: -a for append to shell>")
        exit()

    path = sys.argv[1]
    create_new_crate(path)

    if len(sys.argv) == 3:
        append_to_shell_config(os.path.abspath(path))


def create_new_crate(crate_name):
    # Run `cargo new` command
    subprocess.run(['cargo', 'new', '--lib', crate_name])

    # Change directory to the newly created crate
    os.chdir(crate_name)

    # Add `anyhow` crate as a dependency in Cargo.toml
    with open('Cargo.toml', 'a') as cargo_file:
        cargo_file.write(CARGO_TOML_TEXT)

    # Create .cargo/
    config_dir = os.path.join('.cargo')
    if not os.path.exists(config_dir):
        os.makedirs(config_dir)

    config_file_path = os.path.join(config_dir, 'config.toml')

    # Set the default target to wasm32-unknown-unknown
    with open(config_file_path, 'w') as config_file:
        config_file.write(CONFIG_TOML_TEXT)

    # Write default lib.rs
    lib_rs_path = os.path.join('src/lib.rs')
    with open(lib_rs_path, 'w') as lib_rs:
        lib_rs.write(LIB_RS_TEXT)


    print(f'Created new crate "{crate_name}" with dependencies and config.toml')


def append_to_shell_config(new_line):
    SUPPORTED_SHELLS = {
        "Linux": {
            "config_file": os.path.join(os.path.expanduser("~"), ".bashrc"),
            "export_line": 'export CIMVR_PLUGINS="$CIMVR_PLUGINS;{new_line}"'
        },
        #  UNTESTED IMPL WRITTEN BY CHATGPT BEWARE!
        # "Windows": {
        #     "config_file": os.path.join(os.path.expanduser("~"), "Documents", "WindowsPowerShell", "Microsoft.PowerShell_profile.ps1"),
        #     "export_line": '$env:CIMVR_PLUGINS="$env:CIMVR_PLUGINS;{new_line}"'
        # },
    }

    # TODO: This is probably considered rude...
    system = platform.system()

    if system not in SUPPORTED_SHELLS:
        print("Unsupported operating system.")
        return

    shell = SUPPORTED_SHELLS[system]
    shell_config = shell["config_file"]
    export_line = shell["export_line"].format(new_line=new_line)

    with open(shell_config, "a") as config_file:
        config_file.write(export_line)

    print(f"Added the line '{export_line}' to the shell configuration file.")


CONFIG_TOML_TEXT = """# Written by new.py
[build]
target = "wasm32-unknown-unknown"

[alias]
test_pc = "test --target=x86_64-unknown-linux-gnu"
"""

CARGO_TOML_TEXT = """# Written by new.py
cimvr_common = { git = "https://github.com/ChatImproVR/iteration0.git", branch = "main" }
cimvr_engine_interface  = { git = "https://github.com/ChatImproVR/iteration0.git", branch = "main" }
serde = { version = "1", features = ["derive"] }

[lib]
crate-type = ["cdylib"]
"""

LIB_RS_TEXT = """// Written by new.py, with love
use cimvr_engine_interface::{make_app_state, prelude::*, println};

// All state associated with client-side behaviour
struct ClientState;

impl UserState for ClientState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        println!("Hello, client!");

        // NOTE: We are using the println defined by cimvr_engine_interface here, NOT the standard library!
        cimvr_engine_interface::println!("This prints");
        std::println!("But this doesn't");

        Self
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        println!("Hello, server!");
        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);
"""

if __name__ == "__main__":
    main()
