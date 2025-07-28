{
  description = "Rust project with optional vendored deps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      rust = pkgs.rust-bin.stable.latest.default;
      ps = [
        rust
        pkgs.pkg-config
        pkgs.openssl
      ];

      vendorAvailable = builtins.pathExists ./vendor;

      cargoConfig =
        if vendorAvailable
        then
          pkgs.writeTextDir ".cargo/config.toml" ''
            [source.crates-io]
            replace-with = "vendored-sources"

            [source.vendored-sources]
            directory = "vendor"
          ''
        else null;
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "my-rust-app";
        version = "0.1.0";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        inherit cargoConfig;
        CARGO_HOME =
          if vendorAvailable
          then "${cargoConfig}/.cargo"
          else null;
        cargoVendorDir =
          if vendorAvailable
          then ./vendor
          else null;
        nativeBuildInputs = ps;
        buildInputs = [pkgs.openssl];
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };

      devShells.default = pkgs.mkShell {
        name = "rust-dev-shell";
        nativeBuildInputs =
          ps
          ++ (
            if vendorAvailable
            then [pkgs.cacert]
            else []
          );

        shellHook =
          if vendorAvailable
          then ''
            export CARGO_HOME=${cargoConfig}/.cargo
            echo "🦀 Using vendored Rust dependencies from ./vendor"
          ''
          else ''
            echo "🦀 No vendor directory found. Falling back to crates.io"
          '';

        # Ensure devshell finds OpenSSL headers
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
    });
}
