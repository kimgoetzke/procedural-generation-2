{
  # Thank you, https://github.com/loophp/rust-shell! Most of this is a copy of it.
  description = "A Rust development shells that's also Bevy-ready";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      perSystem = {
        config,
        pkgs,
        system,
        ...
      }: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };
        makeRustInfo = {
          version,
          profile,
        }: let
          rust = pkgs.rust-bin.${version}.latest.${profile}.override {extensions = ["rust-src"];};
        in {
          name = "rust-" + version + "-" + profile;
          path = "${rust}/lib/rustlib/src/rust/library";
          drvs = [
            pkgs.just
            pkgs.openssl
            pkgs.pkg-config
            pkgs.rust-analyzer
            rust
            pkgs.udev
            pkgs.libudev-zero
            pkgs.alsa-lib
            pkgs.vulkan-loader
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.glibc.dev
            pkgs.libGL
            pkgs.audacity
          ];
        };

        makeRustEnv = {
          version,
          profile,
        }: let
          rustInfo = makeRustInfo {
            inherit version profile;
          };
        in
          pkgs.buildEnv {
            name = rustInfo.name;
            paths = rustInfo.drvs;
          };

        matrix = {
          stable-default = {
            version = "stable";
            profile = "default";
          };
          stable-minimal = {
            version = "stable";
            profile = "minimal";
          };
          beta-default = {
            version = "beta";
            profile = "default";
          };
          beta-minimal = {
            version = "beta";
            profile = "minimal";
          };
          nightly-default = {
            version = "nightly";
            profile = "default";
          };
          nightly-minimal = {
            version = "nightly";
            profile = "minimal";
          };
        };
      in {
        formatter = pkgs.alejandra;
        devShells =
          builtins.mapAttrs
          (
            name: value: let
              version = value.version;
              profile = value.profile;
              rustInfo = makeRustInfo {
                inherit version profile;
              };
            in
              pkgs.mkShell {
                name = rustInfo.name;
                RUST_SRC_PATH = rustInfo.path;
                buildInputs = rustInfo.drvs;
              }
          )
          matrix
          // {
            default = let
              version = matrix.stable-default.version;
              profile = matrix.stable-default.profile;
              rustInfo = makeRustInfo {
                inherit version profile;
              };
            in
              pkgs.mkShell {
                name = rustInfo.name;
                RUST_SRC_PATH = rustInfo.path;
                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath rustInfo.drvs;
                buildInputs = rustInfo.drvs;
                shellHook = ''
                  export CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true
                  export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
                  export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
                  echo ""
                  echo "Welcome to your (Bevy-ready) Rust development environment!" | ${pkgs.lolcat}/bin/lolcat
                  echo ""
                '';
              };
          };
      };
    };
}