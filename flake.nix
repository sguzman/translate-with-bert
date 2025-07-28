{
  description = "Rust project with optional vendored deps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rust = pkgs.rust-bin.stable.latest.default;

        # Check for vendor directory
        vendorAvailable = builtins.pathExists ./vendor;

        # Provide a config.toml if vendored
        cargoConfig = if vendorAvailable then
          pkgs.writeTextDir ".cargo/config.toml" ''
            [source.crates-io]
            replace-with = "vendored-sources"

            [source.vendored-sources]
            directory = "vendor"
          ''
        else
          null;

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "my-rust-app";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit cargoConfig;

          nativeBuildInputs = [ rust ]
            ++ (if vendorAvailable then [ pkgs.cacert ] else []);

          # Optional: Pass vendor path
          CARGO_HOME = if vendorAvailable then "${cargoConfig}/.cargo" else null;

          # Optional override for cargoVendorDir
          cargoVendorDir = if vendorAvailable then ./vendor else null;
        };
      });
}

