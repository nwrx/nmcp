{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:

    # Use flake-utils to generate outputs for all default systems
    # (e.g. x86_64-linux, aarch64-linux, etc.)
    flake-utils.lib.eachDefaultSystem (
      system:
      let

        # Import the nixpkgs package set for the current system
        # and apply the rust overlay to it to get the latest Rust toolchain.
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Import the crane library for building Rust packages
        # and override the toolchain to use the stable Rust version
        # with the x86_64-unknown-linux-musl target.
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "x86_64-unknown-linux-musl" ];
          }
        );

        # Common derivation arguments used for all builds
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          # Include all the necessary container tools in the build environment
          buildInputs = [
            pkgs.podman
            pkgs.podman-compose
            pkgs.runc
            pkgs.conmon
            pkgs.skopeo
            pkgs.fuse-overlayfs
            pkgs.slirp4netns
          ];

          # Skip running tests during the build phase - we'll run them separately
          # with proper Podman configuration
          doCheck = false;

          # Set the target architecture and Rust flags for static linking
          # and to avoid compilation errors due to large stack frames
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

          # Increase `rustc` stack size to avoid compilation errors
          # due to large stack frames in `cargo` dependencies.
          RUST_MIN_STACK = "16777216";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "nmcp-deps";
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        nmcp = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Run clippy (and deny all warnings) on the crate source,
        # reusing the dependency artifacts (e.g. from build scripts or
        # proc-macros) from above.
        #
        # Note that this is done as a separate derivation so it
        # does not impact building just the crate by itself.
        nmcpClippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

        # Define the docker image that simply runs the `nmcp` binary
        # in a container. This is useful for running the binary
        nmcpDockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "nmcp";
          tag = "latest";
          contents = [ nmcp ];
          config.Entrypoint = [ "${nmcp}/bin/nmcp" ];
          config.ExposedPorts = { "8080/tcp" = {}; };
        };
      in
    {

      # Expose the `nmcp` package and the `nmcp` Docker image.
      packages.default = nmcp;
      packages.docker = nmcpDockerImage;

      # Enable devshell that inherits from the `nmcp` package.
      devShells.default = craneLib.devShell {
        inputsFrom = [ nmcp ];
        packages = [
          pkgs.cargo-audit
          pkgs.cargo-watch
          pkgs.podman
          pkgs.podman-compose
        ];
      };

      checks = {
        inherit
          nmcp
          nmcpClippy;
      };
    });
}
