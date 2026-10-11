# How to develop

## Using Nix Flakes, JetBrains RustRover & Direnv

You can run this project in any way you like, but I have set things up to make it easy to develop using JetBrains
RustRover. For this, you'll need:

- `direnv`
- Any Direnv integration plugin e.g. https://plugins.jetbrains.com/plugin/15285-direnv-integration
- `nix`

This way, you'll just need to `direnv allow` in the project directory after which all prerequisites (incl. Rust, Cargo,
all Bevy dependencies, etc.) will be available to you. The JetBrains plugin will ensure that the environment is
available to your IDE and you can run the project from there (vs `cargo build` and `cargo run` in the terminal).

## Reminders

### Using Nix Flakes

Without `direnv`, you can use the Nix Flake by running `nix develop` in the project directory. If you want to use an IDE
such as JetBrains RustRover, you'll have to set up the environment manually. You'll most likely have to make
`LD_LIBRARY_PATH` available to your IDE.

Upgrade the flake by running `nix flake update` in the repository's base directory.

### How to deal with RustRover making problems again

RustRover forgetting where the Rust standard library is?

```
find /nix/store -type d -name rust_lib_src
```

### How to use cargo-flamegraph

- Run the command below to generate a flame graph
    - Linux:
      ```shell
      CARGO_PROFILE_RELEASE_DEBUG=true RUSTFLAGS='-C force-frame-pointers=y' cargo flamegraph -c "record -g" --package=procedural-generation-2 --bin=procedural-generation-2
      ```
    - Windows:
      ```pwsh
      $env:CARGO_PROFILE_RELEASE_DEBUG = "true"; $env:RUSTFLAGS = "-C force-frame-pointers=y"; cargo flamegraph -c "record -g" --package=procedural-generation-2 --bin=procedural-generation-2
      ````
- This should run the application - once you close it, a `flamegraph.svg` will be generated at the root of the
  repository
- Open it in your browser to see the flame graph

### Run configurations

The `.run` folder contains a few run configurations for RustRover. Alternatively, you may want to consider creating:

- A run configuration with environment variable `RUST_LOG=procedural_generation_2=debug` for debug logs
- A run configuration that also appends
  `,procedural_generation_2::generation::object=trace,procedural_generation_2::generation::path=trace` to `RUST_LOG` for
  WFC and pathfinding trace logs
- A run configuration with environment variable `RUST_LOG=bevy_ecs=debug` to see Bevy ECS logs (e.g. which system
  caused an `error[B0003]`)
